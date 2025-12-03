//! Vulkan window wrapper
//!
//! Provides VulkanWindow as an equivalent to EglWindow for the Vulkan backend.

use super::{
    CommandManager, FrameSync, RenderPipeline, VulkanContext, VulkanDevice, VulkanSurface,
    VulkanTexture, MAX_FRAMES_IN_FLIGHT,
};
use anyhow::{Context, Result};
use ash::vk;
use std::sync::Arc;
use tracing::{debug, info};

/// Vulkan window wrapper (analogous to EglWindow)
///
/// Encapsulates all Vulkan resources needed for rendering to a single output.
pub struct VulkanWindow {
    /// Vulkan surface and swapchain
    surface: VulkanSurface,
    /// Render pipeline
    pipeline: RenderPipeline,
    /// Command manager
    commands: CommandManager,
    /// Frame synchronization
    frame_sync: FrameSync,
    /// Framebuffers (one per swapchain image)
    framebuffers: Vec<vk::Framebuffer>,
    /// Descriptor pool
    descriptor_pool: vk::DescriptorPool,
    /// Descriptor sets (one per frame in flight)
    descriptor_sets: Vec<vk::DescriptorSet>,
    /// Current texture (if any)
    texture: Option<VulkanTexture>,
    /// Device reference
    device: Arc<VulkanDevice>,
    /// Window dimensions
    width: i32,
    height: i32,
}

impl VulkanWindow {
    /// Create a new Vulkan window
    pub fn new(ctx: &VulkanContext, surface: VulkanSurface) -> Result<Self> {
        let device = ctx.device().clone();
        let (width, height) = surface.dimensions();

        // Get swapchain format
        let format = surface
            .format()
            .ok_or_else(|| anyhow::anyhow!("Swapchain not initialized"))?;

        // Create render pipeline
        let pipeline = RenderPipeline::new(device.clone(), format)
            .context("Failed to create render pipeline")?;

        // Create framebuffers
        let framebuffers = Self::create_framebuffers(&device, &surface, &pipeline)?;

        // Create command manager
        let commands = CommandManager::new(device.clone(), framebuffers.len() as u32)
            .context("Failed to create command manager")?;

        // Create frame synchronization
        let frame_sync = FrameSync::new(device.clone()).context("Failed to create frame sync")?;

        // Create descriptor pool and sets
        let (descriptor_pool, descriptor_sets) = Self::create_descriptors(&device, &pipeline)?;

        info!(
            "VulkanWindow created: {}x{}, {} framebuffers",
            width,
            height,
            framebuffers.len()
        );

        Ok(Self {
            surface,
            pipeline,
            commands,
            frame_sync,
            framebuffers,
            descriptor_pool,
            descriptor_sets,
            texture: None,
            device,
            width: width as i32,
            height: height as i32,
        })
    }

    /// Create framebuffers for swapchain images
    fn create_framebuffers(
        _device: &VulkanDevice,
        surface: &VulkanSurface,
        pipeline: &RenderPipeline,
    ) -> Result<Vec<vk::Framebuffer>> {
        let extent = surface
            .extent()
            .ok_or_else(|| anyhow::anyhow!("Swapchain not initialized"))?;

        // Get image views from surface
        // Note: This requires access to swapchain internal image views
        // For now, we create a single framebuffer per frame
        let mut framebuffers = Vec::new();

        // We need to iterate over swapchain image views
        // Since VulkanSurface only exposes current_image_view(), we'll create
        // framebuffers lazily or assume a fixed count
        for _ in 0..3 {
            // Assume triple buffering
            // Create a placeholder framebuffer that will be recreated on resize
            let _framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(pipeline.render_pass())
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            // Note: In a real implementation, we'd attach the actual image view here
            // For now, we'll handle this in render() by using the current image view
            let fb = vk::Framebuffer::null();
            framebuffers.push(fb);
        }

        Ok(framebuffers)
    }

    /// Create descriptor pool and sets
    fn create_descriptors(
        device: &VulkanDevice,
        pipeline: &RenderPipeline,
    ) -> Result<(vk::DescriptorPool, Vec<vk::DescriptorSet>)> {
        // Create descriptor pool
        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32);

        let pool = unsafe { device.handle().create_descriptor_pool(&pool_info, None) }
            .context("Failed to create descriptor pool")?;

        // Allocate descriptor sets
        let layouts = vec![pipeline.descriptor_set_layout(); MAX_FRAMES_IN_FLIGHT];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);

        let sets = unsafe { device.handle().allocate_descriptor_sets(&alloc_info) }
            .context("Failed to allocate descriptor sets")?;

        Ok((pool, sets))
    }

    /// Update descriptor set with texture
    fn update_descriptor(&self, frame_index: usize, texture: &VulkanTexture) {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture.view())
            .sampler(texture.sampler());

        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_sets[frame_index])
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));

        unsafe {
            self.device.handle().update_descriptor_sets(&[write], &[]);
        }
    }

    /// Resize the window
    pub fn resize(&mut self, width: i32, height: i32) -> Result<()> {
        if self.width == width && self.height == height {
            return Ok(());
        }

        info!(
            "VulkanWindow resize: {}x{} -> {}x{}",
            self.width, self.height, width, height
        );

        self.width = width;
        self.height = height;

        // Resize surface (recreates swapchain)
        self.surface.resize(width as u32, height as u32)?;

        // Recreate framebuffers
        self.cleanup_framebuffers();
        self.framebuffers = Self::create_framebuffers(&self.device, &self.surface, &self.pipeline)?;

        Ok(())
    }

    /// Clean up framebuffers
    fn cleanup_framebuffers(&mut self) {
        unsafe {
            for fb in &self.framebuffers {
                if *fb != vk::Framebuffer::null() {
                    self.device.handle().destroy_framebuffer(*fb, None);
                }
            }
        }
        self.framebuffers.clear();
    }

    /// Set texture for rendering
    pub fn set_texture(&mut self, texture: VulkanTexture) {
        // Update all descriptor sets
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            self.update_descriptor(i, &texture);
        }
        self.texture = Some(texture);
    }

    /// Render a frame
    ///
    /// Returns true if a frame was rendered.
    pub fn render(&mut self) -> Result<bool> {
        // Check if we have a texture to render
        if self.texture.is_none() {
            return Ok(false);
        }

        // Wait for previous frame
        self.frame_sync.wait_for_frame()?;
        self.frame_sync.reset_fence()?;

        // Acquire next image
        let _image_index = match self.surface.acquire_next_image() {
            Ok(idx) => idx,
            Err(e) => {
                debug!("Failed to acquire image, may need resize: {}", e);
                return Ok(false);
            }
        };

        let frame_index = self.frame_sync.current_frame();
        let extent = self.surface.extent().unwrap_or(vk::Extent2D {
            width: self.width as u32,
            height: self.height as u32,
        });

        // Get current image view and create/update framebuffer
        if let Some(image_view) = self.surface.current_image_view() {
            // Create framebuffer for this image
            let attachments = [image_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(self.pipeline.render_pass())
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            let framebuffer = unsafe {
                self.device
                    .handle()
                    .create_framebuffer(&framebuffer_info, None)
            }
            .context("Failed to create framebuffer")?;

            // Record command buffer
            self.commands.record_frame(
                frame_index,
                &self.pipeline,
                extent,
                framebuffer,
                self.descriptor_sets[frame_index],
            )?;

            // Get sync objects
            let (image_available, render_finished, in_flight) =
                self.surface.sync_objects().unwrap();

            // Submit command buffer
            let command_buffer = self.commands.get_buffer(frame_index);
            let wait_semaphores = [image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [render_finished];
            let command_buffers = [command_buffer];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            unsafe {
                self.device.handle().queue_submit(
                    self.device.graphics_queue(),
                    &[submit_info],
                    in_flight,
                )
            }
            .context("Failed to submit draw command")?;

            // Present
            self.surface.present()?;

            // Cleanup temporary framebuffer
            unsafe {
                self.device.handle().destroy_framebuffer(framebuffer, None);
            }
        }

        // Advance to next frame
        self.frame_sync.advance_frame();

        Ok(true)
    }

    /// Get window dimensions
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    /// Swap buffers (compatibility with EglWindow interface)
    pub fn swap_buffers(&self) -> Result<()> {
        // In Vulkan, this is handled in render()
        Ok(())
    }
}

impl Drop for VulkanWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.wait_idle();
            self.cleanup_framebuffers();
            self.device
                .handle()
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
        debug!("VulkanWindow destroyed");
    }
}
