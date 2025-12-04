//! Vulkan rendering backend for wayvid
//!
//! This module provides Vulkan-based rendering as an alternative to OpenGL (EGL).
//! It supports Wayland surfaces via VK_KHR_wayland_surface extension.

// Allow dead code and unused re-exports as they are part of the public API
#![allow(dead_code)]
#![allow(unused_imports)]

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
use ash::vk;
use std::ffi::CStr;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Information about Vulkan availability
#[derive(Debug, Clone, Default)]
pub struct VulkanInfo {
    /// Whether Vulkan is available on this system
    pub available: bool,
    /// Vulkan API version (if available)
    pub api_version: Option<String>,
    /// GPU name (if available)
    pub gpu_name: Option<String>,
    /// Driver version (if available)
    pub driver_version: Option<String>,
    /// Whether required Wayland extensions are supported
    pub wayland_supported: bool,
    /// Error message if Vulkan is not available
    pub error: Option<String>,
}

/// Check if Vulkan is available and get system information
///
/// This function performs a lightweight check without creating a full context.
/// It can be called from the GUI to determine if Vulkan is a valid option.
pub fn check_vulkan_availability() -> VulkanInfo {
    let mut info = VulkanInfo::default();

    // Try to load Vulkan library
    let entry = match unsafe { ash::Entry::load() } {
        Ok(e) => e,
        Err(e) => {
            info.error = Some(format!("Failed to load Vulkan library: {}", e));
            return info;
        }
    };

    // Check API version
    let api_version = match unsafe { entry.try_enumerate_instance_version() } {
        Ok(Some(version)) => version,
        Ok(None) => vk::API_VERSION_1_0,
        Err(e) => {
            info.error = Some(format!("Failed to query Vulkan version: {:?}", e));
            return info;
        }
    };

    info.api_version = Some(format!(
        "{}.{}.{}",
        vk::api_version_major(api_version),
        vk::api_version_minor(api_version),
        vk::api_version_patch(api_version)
    ));

    // Check required extensions for Wayland
    let extensions = match unsafe { entry.enumerate_instance_extension_properties(None) } {
        Ok(exts) => exts,
        Err(e) => {
            info.error = Some(format!("Failed to enumerate extensions: {:?}", e));
            return info;
        }
    };

    let has_surface = extensions.iter().any(|ext| {
        let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
        name == ash::khr::surface::NAME
    });

    let has_wayland_surface = extensions.iter().any(|ext| {
        let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
        name == ash::khr::wayland_surface::NAME
    });

    info.wayland_supported = has_surface && has_wayland_surface;

    if !info.wayland_supported {
        info.error = Some(format!(
            "Missing required Vulkan extensions: VK_KHR_surface={}, VK_KHR_wayland_surface={}",
            has_surface, has_wayland_surface
        ));
        return info;
    }

    // Try to create a minimal instance to query GPU info
    let app_name = std::ffi::CString::new("wayvid-check").unwrap();
    let app_info = vk::ApplicationInfo::default()
        .application_name(&app_name)
        .api_version(vk::API_VERSION_1_0);

    let extensions_ptrs: Vec<*const i8> = vec![
        ash::khr::surface::NAME.as_ptr(),
        ash::khr::wayland_surface::NAME.as_ptr(),
    ];

    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&extensions_ptrs);

    let instance = match unsafe { entry.create_instance(&create_info, None) } {
        Ok(inst) => inst,
        Err(e) => {
            info.error = Some(format!("Failed to create Vulkan instance: {:?}", e));
            return info;
        }
    };

    // Enumerate physical devices
    let physical_devices = match unsafe { instance.enumerate_physical_devices() } {
        Ok(devs) => devs,
        Err(e) => {
            unsafe { instance.destroy_instance(None) };
            info.error = Some(format!("Failed to enumerate GPUs: {:?}", e));
            return info;
        }
    };

    if physical_devices.is_empty() {
        unsafe { instance.destroy_instance(None) };
        info.error = Some("No Vulkan-capable GPU found".to_string());
        return info;
    }

    // Get info from first GPU
    let props = unsafe { instance.get_physical_device_properties(physical_devices[0]) };

    info.gpu_name = Some(
        unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
            .to_string_lossy()
            .to_string(),
    );

    let driver_version = props.driver_version;
    info.driver_version = Some(format!(
        "{}.{}.{}",
        vk::api_version_major(driver_version),
        vk::api_version_minor(driver_version),
        vk::api_version_patch(driver_version)
    ));

    // Cleanup
    unsafe { instance.destroy_instance(None) };

    info.available = true;
    info
}

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
