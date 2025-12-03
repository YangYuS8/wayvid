//! Vulkan command buffer management
//!
//! Handles command pool and buffer allocation for rendering commands.

use super::{RenderPipeline, VulkanDevice};
use anyhow::{Context, Result};
use ash::vk;
use std::sync::Arc;
use tracing::debug;

/// Command buffer manager
pub struct CommandManager {
    /// Reference to device
    device: Arc<VulkanDevice>,
    /// Command pool
    command_pool: vk::CommandPool,
    /// Command buffers (one per swapchain image)
    command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandManager {
    /// Create a new command manager
    pub fn new(device: Arc<VulkanDevice>, image_count: u32) -> Result<Self> {
        // Create command pool
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(device.queue_families().graphics.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe { device.handle().create_command_pool(&pool_info, None) }
            .context("Failed to create command pool")?;

        // Allocate command buffers
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(image_count);

        let command_buffers = unsafe { device.handle().allocate_command_buffers(&alloc_info) }
            .context("Failed to allocate command buffers")?;

        debug!(
            "Command manager created with {} buffers",
            command_buffers.len()
        );

        Ok(Self {
            device,
            command_pool,
            command_buffers,
        })
    }

    /// Get command buffer for a given frame index
    pub fn get_buffer(&self, index: usize) -> vk::CommandBuffer {
        self.command_buffers[index]
    }

    /// Record rendering commands for a frame
    pub fn record_frame(
        &self,
        index: usize,
        pipeline: &RenderPipeline,
        extent: vk::Extent2D,
        framebuffer: vk::Framebuffer,
        descriptor_set: vk::DescriptorSet,
    ) -> Result<()> {
        let command_buffer = self.command_buffers[index];

        // Reset command buffer
        unsafe {
            self.device
                .handle()
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
        }
        .context("Failed to reset command buffer")?;

        // Begin recording
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .handle()
                .begin_command_buffer(command_buffer, &begin_info)
        }
        .context("Failed to begin command buffer")?;

        // Begin render pass
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(pipeline.render_pass())
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.handle().cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            // Bind pipeline
            self.device.handle().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline(),
            );

            // Set dynamic viewport
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            self.device
                .handle()
                .cmd_set_viewport(command_buffer, 0, &[viewport]);

            // Set dynamic scissor
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };
            self.device
                .handle()
                .cmd_set_scissor(command_buffer, 0, &[scissor]);

            // Bind descriptor set (texture)
            self.device.handle().cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline_layout(),
                0,
                &[descriptor_set],
                &[],
            );

            // Draw fullscreen quad (6 vertices for 2 triangles)
            self.device.handle().cmd_draw(command_buffer, 6, 1, 0, 0);

            // End render pass
            self.device.handle().cmd_end_render_pass(command_buffer);
        }

        // End recording
        unsafe { self.device.handle().end_command_buffer(command_buffer) }
            .context("Failed to end command buffer")?;

        Ok(())
    }

    /// Record commands with push constants
    pub fn record_frame_with_push_constants<T: Copy>(
        &self,
        index: usize,
        pipeline: &RenderPipeline,
        extent: vk::Extent2D,
        framebuffer: vk::Framebuffer,
        descriptor_set: vk::DescriptorSet,
        push_constants: &T,
    ) -> Result<()> {
        let command_buffer = self.command_buffers[index];

        unsafe {
            self.device
                .handle()
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;
        }

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .handle()
                .begin_command_buffer(command_buffer, &begin_info)?;
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(pipeline.render_pass())
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.handle().cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            self.device.handle().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline(),
            );

            // Push constants
            let push_data: &[u8] = std::slice::from_raw_parts(
                push_constants as *const T as *const u8,
                std::mem::size_of::<T>(),
            );
            self.device.handle().cmd_push_constants(
                command_buffer,
                pipeline.pipeline_layout(),
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                push_data,
            );

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            self.device
                .handle()
                .cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };
            self.device
                .handle()
                .cmd_set_scissor(command_buffer, 0, &[scissor]);

            self.device.handle().cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline_layout(),
                0,
                &[descriptor_set],
                &[],
            );

            self.device.handle().cmd_draw(command_buffer, 6, 1, 0, 0);

            self.device.handle().cmd_end_render_pass(command_buffer);
        }

        unsafe { self.device.handle().end_command_buffer(command_buffer) }
            .context("Failed to end command buffer")?;

        Ok(())
    }

    /// Get command pool
    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }
}

impl Drop for CommandManager {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle()
                .destroy_command_pool(self.command_pool, None);
        }
        debug!("Command manager destroyed");
    }
}
