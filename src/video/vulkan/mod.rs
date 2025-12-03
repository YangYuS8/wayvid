//! Vulkan rendering backend for wayvid
//!
//! This module provides Vulkan-based rendering as an alternative to OpenGL (EGL).
//! It supports Wayland surfaces via VK_KHR_wayland_surface extension.

mod command;
mod device;
mod instance;
mod pipeline;
mod scene;
pub mod shaders;
mod surface;
mod sync;
mod texture;
mod window;

pub use command::CommandManager;
pub use device::VulkanDevice;
pub use instance::VulkanInstance;
pub use pipeline::{PushConstants, RenderPipeline};
pub use scene::VulkanSceneRenderer;
pub use surface::VulkanSurface;
pub use sync::{FrameSync, MAX_FRAMES_IN_FLIGHT};
pub use texture::VulkanTexture;
pub use window::VulkanWindow;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

/// Vulkan rendering context (analogous to EglContext)
pub struct VulkanContext {
    /// Vulkan instance
    instance: Arc<VulkanInstance>,
    /// Logical device and queues
    device: Arc<VulkanDevice>,
}

impl VulkanContext {
    /// Create a new Vulkan context for Wayland
    pub fn new(wl_display: *mut std::ffi::c_void) -> Result<Self> {
        info!("Initializing Vulkan context for Wayland");

        // Create Vulkan instance with Wayland surface support
        let instance = Arc::new(VulkanInstance::new()?);
        info!("  ✓ Vulkan instance created");

        // Select physical device and create logical device
        let device = Arc::new(VulkanDevice::new(instance.clone(), wl_display)?);
        info!("  ✓ Vulkan device created");

        Ok(Self { instance, device })
    }

    /// Create a surface for a Wayland surface
    pub fn create_surface(
        &self,
        wl_surface: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<VulkanSurface> {
        VulkanSurface::new(
            self.instance.clone(),
            self.device.clone(),
            wl_surface,
            width,
            height,
        )
    }

    /// Get the Vulkan instance
    pub fn instance(&self) -> &Arc<VulkanInstance> {
        &self.instance
    }

    /// Get the Vulkan device
    pub fn device(&self) -> &Arc<VulkanDevice> {
        &self.device
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        info!("Destroying Vulkan context");
        // Device and instance will be cleaned up when Arc drops
    }
}

// Safety: Vulkan handles are thread-safe when properly synchronized
unsafe impl Send for VulkanContext {}
unsafe impl Sync for VulkanContext {}
