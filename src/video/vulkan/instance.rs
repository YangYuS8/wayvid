//! Vulkan instance management
//!
//! Handles Vulkan instance creation with required extensions for Wayland rendering.

use anyhow::{anyhow, Context, Result};
use ash::vk;
use std::ffi::{CStr, CString};
use tracing::{debug, info, warn};

/// Vulkan instance wrapper
pub struct VulkanInstance {
    /// Ash entry point (library loader)
    entry: ash::Entry,
    /// Vulkan instance handle
    instance: ash::Instance,
    /// Debug messenger (if validation enabled)
    #[cfg(debug_assertions)]
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    #[cfg(debug_assertions)]
    debug_utils: Option<ash::ext::debug_utils::Instance>,
}

impl VulkanInstance {
    /// Required instance extensions for Wayland
    const REQUIRED_EXTENSIONS: &'static [&'static CStr] =
        &[ash::khr::surface::NAME, ash::khr::wayland_surface::NAME];

    /// Create a new Vulkan instance
    pub fn new() -> Result<Self> {
        // Load Vulkan library
        let entry = unsafe { ash::Entry::load() }.context("Failed to load Vulkan library")?;

        // Check API version
        let api_version = match unsafe { entry.try_enumerate_instance_version() }? {
            Some(version) => version,
            None => vk::API_VERSION_1_0,
        };

        info!(
            "Vulkan API version: {}.{}.{}",
            vk::api_version_major(api_version),
            vk::api_version_minor(api_version),
            vk::api_version_patch(api_version)
        );

        // Check required extensions
        let available_extensions = unsafe { entry.enumerate_instance_extension_properties(None) }?;
        debug!("Available instance extensions:");
        for ext in &available_extensions {
            let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            debug!("  - {}", name.to_string_lossy());
        }

        // Verify required extensions are available
        for required in Self::REQUIRED_EXTENSIONS {
            let found = available_extensions.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                name == *required
            });
            if !found {
                return Err(anyhow!(
                    "Required Vulkan extension not available: {}",
                    required.to_string_lossy()
                ));
            }
        }

        // Build extension list
        let mut extensions: Vec<*const i8> = Self::REQUIRED_EXTENSIONS
            .iter()
            .map(|e| e.as_ptr())
            .collect();

        // Add debug extension in debug builds
        #[cfg(debug_assertions)]
        let enable_validation = std::env::var("WAYVID_VULKAN_VALIDATION")
            .map(|v| v == "1")
            .unwrap_or(false);
        #[cfg(not(debug_assertions))]
        let enable_validation = false;

        #[cfg(debug_assertions)]
        if enable_validation {
            extensions.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        // Validation layers
        let layer_names: Vec<CString> = if enable_validation {
            vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()]
        } else {
            vec![]
        };
        let layer_ptrs: Vec<*const i8> = layer_names.iter().map(|l| l.as_ptr()).collect();

        // Application info
        let app_name = CString::new("wayvid").unwrap();
        let engine_name = CString::new("wayvid-vulkan").unwrap();

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 4, 4))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_0);

        // Create instance
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layer_ptrs);

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .context("Failed to create Vulkan instance")?;

        info!("Vulkan instance created");

        // Set up debug messenger
        #[cfg(debug_assertions)]
        let (debug_utils, debug_messenger) = if enable_validation {
            let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);

            let messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let messenger =
                unsafe { debug_utils.create_debug_utils_messenger(&messenger_info, None) }.ok();

            if messenger.is_some() {
                info!("Vulkan validation layer enabled");
            }

            (Some(debug_utils), messenger)
        } else {
            (None, None)
        };

        Ok(Self {
            entry,
            instance,
            #[cfg(debug_assertions)]
            debug_messenger,
            #[cfg(debug_assertions)]
            debug_utils,
        })
    }

    /// Get the ash Entry
    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    /// Get the Vulkan instance handle
    pub fn handle(&self) -> &ash::Instance {
        &self.instance
    }

    /// Get the raw Vulkan instance handle
    pub fn raw(&self) -> vk::Instance {
        self.instance.handle()
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            #[cfg(debug_assertions)]
            if let (Some(ref debug_utils), Some(messenger)) =
                (&self.debug_utils, self.debug_messenger)
            {
                debug_utils.destroy_debug_utils_messenger(messenger, None);
            }

            self.instance.destroy_instance(None);
        }
        debug!("Vulkan instance destroyed");
    }
}

/// Vulkan debug callback
#[cfg(debug_assertions)]
unsafe extern "system" fn vulkan_debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = if !callback_data.is_null() {
        let data = &*callback_data;
        if !data.p_message.is_null() {
            CStr::from_ptr(data.p_message).to_string_lossy()
        } else {
            std::borrow::Cow::Borrowed("(no message)")
        }
    } else {
        std::borrow::Cow::Borrowed("(no data)")
    };

    if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
        tracing::error!("[Vulkan] {}", message);
    } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
        warn!("[Vulkan] {}", message);
    } else {
        debug!("[Vulkan] {}", message);
    }

    vk::FALSE
}
