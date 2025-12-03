//! Vulkan surface and swapchain management
//!
//! Handles Wayland surface integration and swapchain creation.

use super::{VulkanDevice, VulkanInstance};
use anyhow::{anyhow, Context, Result};
use ash::vk;
use std::sync::Arc;
use tracing::{debug, info};

/// Vulkan surface wrapper (analogous to EglWindow)
pub struct VulkanSurface {
    /// Reference to instance
    instance: Arc<VulkanInstance>,
    /// Reference to device
    device: Arc<VulkanDevice>,
    /// Vulkan surface handle
    surface: vk::SurfaceKHR,
    /// Surface extension loader
    surface_loader: ash::khr::surface::Instance,
    /// Swapchain
    swapchain: Option<SwapchainData>,
    /// Surface dimensions
    width: u32,
    height: u32,
}

/// Swapchain data
struct SwapchainData {
    /// Swapchain handle
    swapchain: vk::SwapchainKHR,
    /// Swapchain extension loader
    swapchain_loader: ash::khr::swapchain::Device,
    /// Swapchain images
    images: Vec<vk::Image>,
    /// Swapchain image views
    image_views: Vec<vk::ImageView>,
    /// Swapchain format
    format: vk::SurfaceFormatKHR,
    /// Swapchain extent
    extent: vk::Extent2D,
    /// Current image index
    current_image: u32,
    /// Image available semaphore
    image_available: vk::Semaphore,
    /// Render finished semaphore
    render_finished: vk::Semaphore,
    /// In-flight fence
    in_flight: vk::Fence,
}

impl VulkanSurface {
    /// Create a new Vulkan surface from a Wayland surface
    pub fn new(
        instance: Arc<VulkanInstance>,
        device: Arc<VulkanDevice>,
        wl_surface: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        // Create surface loader
        let surface_loader = ash::khr::surface::Instance::new(instance.entry(), instance.handle());

        // Create Wayland surface
        let wayland_surface_loader =
            ash::khr::wayland_surface::Instance::new(instance.entry(), instance.handle());

        // Note: wl_display is needed here, but we're simplifying for now
        // In a real implementation, we'd pass wl_display through
        let surface_create_info = vk::WaylandSurfaceCreateInfoKHR::default()
            .display(std::ptr::null_mut()) // This would need the actual wl_display
            .surface(wl_surface as *mut _);

        let surface =
            unsafe { wayland_surface_loader.create_wayland_surface(&surface_create_info, None) }
                .context("Failed to create Wayland Vulkan surface")?;

        info!("Vulkan surface created: {}x{}", width, height);

        let mut vk_surface = Self {
            instance,
            device,
            surface,
            surface_loader,
            swapchain: None,
            width,
            height,
        };

        // Create initial swapchain
        vk_surface.create_swapchain()?;

        Ok(vk_surface)
    }

    /// Create or recreate the swapchain
    fn create_swapchain(&mut self) -> Result<()> {
        // Wait for device to be idle before recreating
        self.device.wait_idle()?;

        // Clean up old swapchain
        if let Some(old_swapchain) = self.swapchain.take() {
            self.cleanup_swapchain(old_swapchain);
        }

        let physical_device = self.device.physical_device();

        // Query surface capabilities
        let capabilities = unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
        .context("Failed to get surface capabilities")?;

        // Query surface formats
        let formats = unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device, self.surface)
        }
        .context("Failed to get surface formats")?;

        // Query present modes
        let present_modes = unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(physical_device, self.surface)
        }
        .context("Failed to get present modes")?;

        // Choose surface format (prefer SRGB)
        let format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .or_else(|| formats.first())
            .copied()
            .ok_or_else(|| anyhow!("No suitable surface format"))?;

        // Choose present mode (prefer mailbox for low latency, fallback to FIFO)
        let present_mode = present_modes
            .iter()
            .find(|&&m| m == vk::PresentModeKHR::MAILBOX)
            .or_else(|| {
                present_modes
                    .iter()
                    .find(|&&m| m == vk::PresentModeKHR::FIFO)
            })
            .copied()
            .unwrap_or(vk::PresentModeKHR::FIFO);

        // Choose extent
        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: self.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: self.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        // Choose image count (prefer triple buffering)
        let image_count =
            (capabilities.min_image_count + 1).min(if capabilities.max_image_count > 0 {
                capabilities.max_image_count
            } else {
                u32::MAX
            });

        // Create swapchain
        let swapchain_loader =
            ash::khr::swapchain::Device::new(self.instance.handle(), self.device.handle());

        let queue_families = self.device.queue_families();
        let queue_family_indices = [
            queue_families.graphics.unwrap(),
            queue_families.present.unwrap(),
        ];

        let sharing_mode = if queue_families.graphics == queue_families.present {
            vk::SharingMode::EXCLUSIVE
        } else {
            vk::SharingMode::CONCURRENT
        };

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(if sharing_mode == vk::SharingMode::CONCURRENT {
                &queue_family_indices
            } else {
                &[]
            })
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }
            .context("Failed to create swapchain")?;

        // Get swapchain images
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }
            .context("Failed to get swapchain images")?;

        // Create image views
        let image_views = images
            .iter()
            .map(|&image| self.create_image_view(image, format.format))
            .collect::<Result<Vec<_>>>()?;

        // Create synchronization objects
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let image_available =
            unsafe { self.device.handle().create_semaphore(&semaphore_info, None) }
                .context("Failed to create image available semaphore")?;

        let render_finished =
            unsafe { self.device.handle().create_semaphore(&semaphore_info, None) }
                .context("Failed to create render finished semaphore")?;

        let in_flight = unsafe { self.device.handle().create_fence(&fence_info, None) }
            .context("Failed to create in-flight fence")?;

        self.swapchain = Some(SwapchainData {
            swapchain,
            swapchain_loader,
            images,
            image_views,
            format,
            extent,
            current_image: 0,
            image_available,
            render_finished,
            in_flight,
        });

        info!(
            "Swapchain created: {}x{}, {} images, {:?}",
            extent.width,
            extent.height,
            self.swapchain.as_ref().unwrap().images.len(),
            present_mode
        );

        Ok(())
    }

    /// Create an image view
    fn create_image_view(&self, image: vk::Image, format: vk::Format) -> Result<vk::ImageView> {
        let create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        unsafe { self.device.handle().create_image_view(&create_info, None) }
            .context("Failed to create image view")
    }

    /// Clean up swapchain resources
    fn cleanup_swapchain(&self, swapchain_data: SwapchainData) {
        unsafe {
            self.device
                .handle()
                .destroy_fence(swapchain_data.in_flight, None);
            self.device
                .handle()
                .destroy_semaphore(swapchain_data.render_finished, None);
            self.device
                .handle()
                .destroy_semaphore(swapchain_data.image_available, None);

            for &view in &swapchain_data.image_views {
                self.device.handle().destroy_image_view(view, None);
            }

            swapchain_data
                .swapchain_loader
                .destroy_swapchain(swapchain_data.swapchain, None);
        }
    }

    /// Acquire next image for rendering
    pub fn acquire_next_image(&mut self) -> Result<u32> {
        let swapchain_data = self
            .swapchain
            .as_mut()
            .ok_or_else(|| anyhow!("Swapchain not initialized"))?;

        // Wait for previous frame
        unsafe {
            self.device
                .handle()
                .wait_for_fences(&[swapchain_data.in_flight], true, u64::MAX)
        }
        .context("Failed to wait for fence")?;

        // Acquire next image
        let (image_index, _suboptimal) = unsafe {
            swapchain_data.swapchain_loader.acquire_next_image(
                swapchain_data.swapchain,
                u64::MAX,
                swapchain_data.image_available,
                vk::Fence::null(),
            )
        }
        .context("Failed to acquire next image")?;

        // Reset fence
        unsafe {
            self.device
                .handle()
                .reset_fences(&[swapchain_data.in_flight])
        }
        .context("Failed to reset fence")?;

        swapchain_data.current_image = image_index;
        Ok(image_index)
    }

    /// Present the rendered image
    pub fn present(&self) -> Result<()> {
        let swapchain_data = self
            .swapchain
            .as_ref()
            .ok_or_else(|| anyhow!("Swapchain not initialized"))?;

        let swapchains = [swapchain_data.swapchain];
        let image_indices = [swapchain_data.current_image];
        let wait_semaphores = [swapchain_data.render_finished];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            swapchain_data
                .swapchain_loader
                .queue_present(self.device.present_queue(), &present_info)
        }
        .context("Failed to present")?;

        Ok(())
    }

    /// Resize the surface
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if self.width == width && self.height == height {
            return Ok(());
        }

        self.width = width;
        self.height = height;

        self.create_swapchain()?;
        debug!("Vulkan surface resized to {}x{}", width, height);

        Ok(())
    }

    /// Get surface dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get current swapchain image view
    pub fn current_image_view(&self) -> Option<vk::ImageView> {
        self.swapchain
            .as_ref()
            .map(|s| s.image_views[s.current_image as usize])
    }

    /// Get swapchain format
    pub fn format(&self) -> Option<vk::Format> {
        self.swapchain.as_ref().map(|s| s.format.format)
    }

    /// Get swapchain extent
    pub fn extent(&self) -> Option<vk::Extent2D> {
        self.swapchain.as_ref().map(|s| s.extent)
    }

    /// Get synchronization semaphores for command submission
    pub fn sync_objects(&self) -> Option<(vk::Semaphore, vk::Semaphore, vk::Fence)> {
        self.swapchain
            .as_ref()
            .map(|s| (s.image_available, s.render_finished, s.in_flight))
    }
}

impl Drop for VulkanSurface {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.wait_idle();

            if let Some(swapchain_data) = self.swapchain.take() {
                self.cleanup_swapchain(swapchain_data);
            }

            self.surface_loader.destroy_surface(self.surface, None);
        }
        debug!("Vulkan surface destroyed");
    }
}
