//! Vulkan device management
//!
//! Handles physical device selection and logical device creation.

use super::VulkanInstance;
use anyhow::{anyhow, Context, Result};
use ash::vk;
use std::ffi::CStr;
use std::sync::Arc;
use tracing::{debug, info};

/// Queue family indices
#[derive(Debug, Clone, Copy, Default)]
pub struct QueueFamilyIndices {
    /// Graphics queue family index
    pub graphics: Option<u32>,
    /// Present queue family index (may be same as graphics)
    pub present: Option<u32>,
}

impl QueueFamilyIndices {
    /// Check if all required queue families are found
    pub fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
}

/// Vulkan device wrapper
pub struct VulkanDevice {
    /// Reference to instance
    instance: Arc<VulkanInstance>,
    /// Physical device
    physical_device: vk::PhysicalDevice,
    /// Logical device
    device: ash::Device,
    /// Queue family indices
    queue_families: QueueFamilyIndices,
    /// Graphics queue
    graphics_queue: vk::Queue,
    /// Present queue (may be same as graphics)
    present_queue: vk::Queue,
    /// Device properties
    properties: vk::PhysicalDeviceProperties,
    /// Memory properties (cached)
    memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl VulkanDevice {
    /// Required device extensions
    const REQUIRED_EXTENSIONS: &'static [&'static CStr] = &[ash::khr::swapchain::NAME];

    /// Create a new Vulkan device
    pub fn new(instance: Arc<VulkanInstance>, _wl_display: *mut std::ffi::c_void) -> Result<Self> {
        // Enumerate physical devices
        let physical_devices = unsafe { instance.handle().enumerate_physical_devices() }
            .context("Failed to enumerate physical devices")?;

        if physical_devices.is_empty() {
            return Err(anyhow!("No Vulkan-capable GPU found"));
        }

        info!("Found {} Vulkan device(s)", physical_devices.len());

        // Select best physical device
        let (physical_device, queue_families, properties) =
            Self::select_physical_device(&instance, &physical_devices)?;

        let device_name =
            unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_string_lossy();
        info!(
            "Selected GPU: {} ({:?})",
            device_name, properties.device_type
        );

        // Create logical device
        let device = Self::create_logical_device(&instance, physical_device, &queue_families)?;

        // Get queues
        let graphics_queue =
            unsafe { device.get_device_queue(queue_families.graphics.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(queue_families.present.unwrap(), 0) };

        // Cache memory properties
        let memory_properties = unsafe {
            instance
                .handle()
                .get_physical_device_memory_properties(physical_device)
        };

        Ok(Self {
            instance,
            physical_device,
            device,
            queue_families,
            graphics_queue,
            present_queue,
            properties,
            memory_properties,
        })
    }

    /// Select the best physical device
    fn select_physical_device(
        instance: &VulkanInstance,
        devices: &[vk::PhysicalDevice],
    ) -> Result<(
        vk::PhysicalDevice,
        QueueFamilyIndices,
        vk::PhysicalDeviceProperties,
    )> {
        let mut best_device = None;
        let mut best_score = 0;

        for &device in devices {
            let properties = unsafe { instance.handle().get_physical_device_properties(device) };
            let features = unsafe { instance.handle().get_physical_device_features(device) };

            let device_name =
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_string_lossy();

            // Check device extensions
            let extensions = unsafe {
                instance
                    .handle()
                    .enumerate_device_extension_properties(device)
            }
            .unwrap_or_default();

            let has_required_extensions = Self::REQUIRED_EXTENSIONS.iter().all(|required| {
                extensions.iter().any(|ext| {
                    let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                    name == *required
                })
            });

            if !has_required_extensions {
                debug!("Device {} missing required extensions", device_name);
                continue;
            }

            // Find queue families
            let queue_families = Self::find_queue_families(instance, device);
            if !queue_families.is_complete() {
                debug!("Device {} missing required queue families", device_name);
                continue;
            }

            // Score device
            let mut score = 0;

            // Prefer discrete GPUs
            if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                score += 1000;
            } else if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
                score += 100;
            }

            // Prefer devices with geometry shader (not strictly required)
            if features.geometry_shader != 0 {
                score += 10;
            }

            // Prefer devices with more VRAM (approximated by max image dimension)
            score += properties.limits.max_image_dimension2_d / 1000;

            debug!(
                "Device {} score: {} (type: {:?})",
                device_name, score, properties.device_type
            );

            if score > best_score {
                best_score = score;
                best_device = Some((device, queue_families, properties));
            }
        }

        best_device.ok_or_else(|| anyhow!("No suitable Vulkan device found"))
    }

    /// Find queue families for a physical device
    fn find_queue_families(
        instance: &VulkanInstance,
        device: vk::PhysicalDevice,
    ) -> QueueFamilyIndices {
        let queue_families = unsafe {
            instance
                .handle()
                .get_physical_device_queue_family_properties(device)
        };

        let mut indices = QueueFamilyIndices::default();

        for (i, family) in queue_families.iter().enumerate() {
            let i = i as u32;

            // Check for graphics support
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics = Some(i);
            }

            // For now, assume present support on same queue as graphics
            // In a real implementation, we'd check surface support
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.present = Some(i);
            }

            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    /// Create logical device
    fn create_logical_device(
        instance: &VulkanInstance,
        physical_device: vk::PhysicalDevice,
        queue_families: &QueueFamilyIndices,
    ) -> Result<ash::Device> {
        // Collect unique queue families
        let mut unique_families = vec![queue_families.graphics.unwrap()];
        if queue_families.present.unwrap() != queue_families.graphics.unwrap() {
            unique_families.push(queue_families.present.unwrap());
        }

        // Create queue create infos
        let queue_priorities = [1.0f32];
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_families
            .iter()
            .map(|&family| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(family)
                    .queue_priorities(&queue_priorities)
            })
            .collect();

        // Device extensions
        let extension_ptrs: Vec<*const i8> = Self::REQUIRED_EXTENSIONS
            .iter()
            .map(|e| e.as_ptr())
            .collect();

        // Device features (minimal for now)
        let features = vk::PhysicalDeviceFeatures::default();

        // Create device
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extension_ptrs)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .handle()
                .create_device(physical_device, &create_info, None)
        }
        .context("Failed to create logical device")?;

        Ok(device)
    }

    /// Get the physical device handle
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    /// Get the logical device handle
    pub fn handle(&self) -> &ash::Device {
        &self.device
    }

    /// Get the graphics queue
    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    /// Get the present queue
    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    /// Get queue family indices
    pub fn queue_families(&self) -> &QueueFamilyIndices {
        &self.queue_families
    }

    /// Get device properties
    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties
    }

    /// Get memory properties
    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }

    /// Get the Vulkan instance
    pub fn instance(&self) -> &Arc<VulkanInstance> {
        &self.instance
    }

    /// Wait for device to be idle
    pub fn wait_idle(&self) -> Result<()> {
        unsafe { self.device.device_wait_idle() }.context("Failed to wait for device idle")?;
        Ok(())
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_device(None);
        }
        debug!("Vulkan device destroyed");
    }
}
