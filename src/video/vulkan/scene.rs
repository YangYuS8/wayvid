//! Vulkan scene renderer
//!
//! Renders Wallpaper Engine scenes using Vulkan backend.

use super::{VulkanDevice, VulkanTexture, VulkanWindow};
use crate::we::scene::{BlendMode, LoadedScene};
use anyhow::{Context, Result};
use ash::vk;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Vulkan scene renderer
pub struct VulkanSceneRenderer {
    /// Loaded scene
    scene: LoadedScene,
    /// Loaded textures (object_id -> VulkanTexture)
    textures: HashMap<u64, VulkanTexture>,
    /// Device reference
    device: Arc<VulkanDevice>,
    /// Current time for animations
    current_time: f64,
    /// Scene width
    width: u32,
    /// Scene height
    height: u32,
    /// Initialized flag
    initialized: bool,
}

impl VulkanSceneRenderer {
    /// Create a new Vulkan scene renderer
    pub fn new(scene: LoadedScene, device: Arc<VulkanDevice>) -> Self {
        let width = scene.resolution.0;
        let height = scene.resolution.1;

        Self {
            scene,
            textures: HashMap::new(),
            device,
            current_time: 0.0,
            width,
            height,
            initialized: false,
        }
    }

    /// Initialize Vulkan resources
    pub fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        info!("Initializing Vulkan scene renderer");

        // Load textures from scene
        // Note: This requires texture data from scene container
        // For now, we mark as initialized and load textures lazily

        self.initialized = true;
        info!(
            "Vulkan scene renderer initialized: {}x{}",
            self.width, self.height
        );

        Ok(())
    }

    /// Load texture from RGBA data
    pub fn load_texture(
        &mut self,
        object_id: u64,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<()> {
        let texture = VulkanTexture::from_rgba(self.device.clone(), width, height, data)
            .with_context(|| format!("Failed to create texture for object {}", object_id))?;

        self.textures.insert(object_id, texture);
        debug!(
            "Loaded Vulkan texture for object {}: {}x{}",
            object_id, width, height
        );

        Ok(())
    }

    /// Get scene resolution
    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get texture count
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Get loaded scene reference
    pub fn scene(&self) -> &LoadedScene {
        &self.scene
    }

    /// Update animation time
    pub fn update(&mut self, delta_time: f64) {
        self.current_time += delta_time;
    }

    /// Render the scene
    ///
    /// Note: Full Vulkan scene rendering requires:
    /// 1. Per-layer pipeline states for blend modes
    /// 2. Multiple render passes or dynamic blending
    /// 3. Transform matrices via push constants
    ///
    /// This is a placeholder implementation that demonstrates the architecture.
    pub fn render(&mut self, window: &mut VulkanWindow) -> Result<bool> {
        if !self.initialized {
            return Ok(false);
        }

        // For now, render the scene by setting textures and calling window.render()
        // A full implementation would:
        // 1. Sort objects by z-order
        // 2. Render each layer with appropriate blend state
        // 3. Apply transforms and effects

        // Get visible objects sorted by z-order
        let mut objects: Vec<_> = self
            .scene
            .scene
            .objects
            .iter()
            .filter(|o| o.visible)
            .collect();
        objects.sort_by(|a, b| {
            a.origin
                .z
                .partial_cmp(&b.origin.z)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Render layers (simplified - full implementation would use multiple draw calls)
        for obj in &objects {
            if let Some(_texture) = self.textures.get(&obj.id) {
                // In a full implementation, we would:
                // 1. Update push constants with object transform
                // 2. Set blend mode for this object
                // 3. Draw the textured quad

                // For now, we just render via the window
            }
        }

        // Delegate to window render (renders first texture)
        window.render()
    }

    /// Get blend factor for Vulkan from scene blend mode
    #[allow(dead_code)]
    fn get_blend_factors(blend_mode: BlendMode) -> (vk::BlendFactor, vk::BlendFactor) {
        match blend_mode {
            BlendMode::Normal => (
                vk::BlendFactor::SRC_ALPHA,
                vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            ),
            BlendMode::Additive => (vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE),
            BlendMode::Multiply => (vk::BlendFactor::DST_COLOR, vk::BlendFactor::ZERO),
            BlendMode::Screen => (vk::BlendFactor::ONE, vk::BlendFactor::ONE_MINUS_SRC_COLOR),
            BlendMode::Overlay => (
                vk::BlendFactor::SRC_ALPHA,
                vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            ),
        }
    }

    /// Create transform matrix for an object
    #[allow(dead_code)]
    fn create_transform_matrix(
        &self,
        origin: (f32, f32, f32),
        scale: (f32, f32),
        angles: (f32, f32, f32),
        viewport_width: u32,
        viewport_height: u32,
    ) -> [[f32; 4]; 4] {
        // Create orthographic projection
        let proj = Self::ortho_matrix(
            0.0,
            viewport_width as f32,
            viewport_height as f32,
            0.0,
            -1.0,
            1.0,
        );

        // Create model matrix (translate, rotate, scale)
        let translate = Self::translation_matrix(origin.0, origin.1, 0.0);
        let rotate = Self::rotation_z_matrix(angles.2.to_radians());
        let scale_mat = Self::scale_matrix(scale.0, scale.1, 1.0);

        // Combine: proj * translate * rotate * scale
        let model = Self::mat4_multiply(&translate, &rotate);
        let model = Self::mat4_multiply(&model, &scale_mat);
        Self::mat4_multiply(&proj, &model)
    }

    /// Create orthographic projection matrix
    fn ortho_matrix(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> [[f32; 4]; 4] {
        let w = right - left;
        let h = top - bottom;
        let d = far - near;
        [
            [2.0 / w, 0.0, 0.0, 0.0],
            [0.0, 2.0 / h, 0.0, 0.0],
            [0.0, 0.0, -2.0 / d, 0.0],
            [
                -(right + left) / w,
                -(top + bottom) / h,
                -(far + near) / d,
                1.0,
            ],
        ]
    }

    /// Create translation matrix
    fn translation_matrix(x: f32, y: f32, z: f32) -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, z, 1.0],
        ]
    }

    /// Create rotation around Z axis
    fn rotation_z_matrix(angle: f32) -> [[f32; 4]; 4] {
        let c = angle.cos();
        let s = angle.sin();
        [
            [c, s, 0.0, 0.0],
            [-s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// Create scale matrix
    fn scale_matrix(x: f32, y: f32, z: f32) -> [[f32; 4]; 4] {
        [
            [x, 0.0, 0.0, 0.0],
            [0.0, y, 0.0, 0.0],
            [0.0, 0.0, z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// Multiply two 4x4 matrices
    fn mat4_multiply(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        result
    }

    /// Cleanup resources
    pub fn cleanup(&mut self) {
        self.textures.clear();
        self.initialized = false;
        debug!("Vulkan scene renderer cleaned up");
    }
}

impl Drop for VulkanSceneRenderer {
    fn drop(&mut self) {
        self.cleanup();
    }
}
