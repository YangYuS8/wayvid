//! Vulkan frame synchronization
//!
//! Manages synchronization primitives for frame rendering.

use super::VulkanDevice;
use anyhow::{Context, Result};
use ash::vk;
use std::sync::Arc;
use tracing::debug;

/// Maximum number of frames in flight
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Frame synchronization primitives
pub struct FrameSync {
    /// Reference to device
    device: Arc<VulkanDevice>,
    /// Semaphores signaled when image is available
    image_available_semaphores: Vec<vk::Semaphore>,
    /// Semaphores signaled when rendering is finished
    render_finished_semaphores: Vec<vk::Semaphore>,
    /// Fences for CPU-GPU synchronization
    in_flight_fences: Vec<vk::Fence>,
    /// Current frame index (0 or 1 for double buffering)
    current_frame: usize,
}

impl FrameSync {
    /// Create synchronization primitives
    pub fn new(device: Arc<VulkanDevice>) -> Result<Self> {
        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let image_available =
                unsafe { device.handle().create_semaphore(&semaphore_info, None) }
                    .context("Failed to create image available semaphore")?;

            let render_finished =
                unsafe { device.handle().create_semaphore(&semaphore_info, None) }
                    .context("Failed to create render finished semaphore")?;

            let fence = unsafe { device.handle().create_fence(&fence_info, None) }
                .context("Failed to create fence")?;

            image_available_semaphores.push(image_available);
            render_finished_semaphores.push(render_finished);
            in_flight_fences.push(fence);
        }

        debug!(
            "Frame sync created with {} frames in flight",
            MAX_FRAMES_IN_FLIGHT
        );

        Ok(Self {
            device,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
        })
    }

    /// Wait for current frame's fence
    pub fn wait_for_frame(&self) -> Result<()> {
        let fence = self.in_flight_fences[self.current_frame];
        unsafe {
            self.device
                .handle()
                .wait_for_fences(&[fence], true, u64::MAX)
        }
        .context("Failed to wait for fence")?;
        Ok(())
    }

    /// Reset current frame's fence
    pub fn reset_fence(&self) -> Result<()> {
        let fence = self.in_flight_fences[self.current_frame];
        unsafe { self.device.handle().reset_fences(&[fence]) }.context("Failed to reset fence")?;
        Ok(())
    }

    /// Get current frame index
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Get image available semaphore for current frame
    pub fn image_available_semaphore(&self) -> vk::Semaphore {
        self.image_available_semaphores[self.current_frame]
    }

    /// Get render finished semaphore for current frame
    pub fn render_finished_semaphore(&self) -> vk::Semaphore {
        self.render_finished_semaphores[self.current_frame]
    }

    /// Get fence for current frame
    pub fn in_flight_fence(&self) -> vk::Fence {
        self.in_flight_fences[self.current_frame]
    }

    /// Advance to next frame
    pub fn advance_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for FrameSync {
    fn drop(&mut self) {
        unsafe {
            for &semaphore in &self.image_available_semaphores {
                self.device.handle().destroy_semaphore(semaphore, None);
            }
            for &semaphore in &self.render_finished_semaphores {
                self.device.handle().destroy_semaphore(semaphore, None);
            }
            for &fence in &self.in_flight_fences {
                self.device.handle().destroy_fence(fence, None);
            }
        }
        debug!("Frame sync destroyed");
    }
}
