//! Scene-based video source for Wallpaper Engine scenes
//!
//! This module integrates scene rendering with the wayvid video playback system.

// Allow dead code for public API items
#![allow(dead_code)]

use crate::core::types::OutputInfo;
use crate::video::egl::{EglContext, EglWindow};
use crate::we::scene::{SceneContainer, SceneParser, SceneRenderer};
use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info};

/// Scene-based wallpaper player
pub struct ScenePlayer {
    /// Scene renderer
    renderer: SceneRenderer,
    /// Last frame time for delta calculation
    last_frame_time: Instant,
    /// Target FPS
    target_fps: u32,
    /// Output info
    output_info: OutputInfo,
    /// Whether initialized
    initialized: bool,
}

impl ScenePlayer {
    /// Create a new scene player from a project directory
    pub fn new(project_path: &Path, output_info: &OutputInfo) -> Result<Self> {
        info!("🎭 Loading scene wallpaper for output {}", output_info.name);

        // Parse the scene
        let project = SceneParser::load(project_path)
            .with_context(|| format!("Failed to load scene from {:?}", project_path))?;

        info!(
            "📦 Scene loaded: '{}' ({} objects, {}x{})",
            project.title,
            project.objects.len(),
            project.resolution.0,
            project.resolution.1
        );

        // Create scene container for PKG/file access
        let container = SceneContainer::new(project_path)
            .with_context(|| format!("Failed to create scene container for {:?}", project_path))?;

        // Create renderer with container for texture loading
        let mut renderer = SceneRenderer::with_container(project.loaded_scene, container);

        // Load texture resources (CPU side - from PKG or filesystem)
        renderer
            .load_resources()
            .context("Failed to load scene resources")?;

        info!(
            "🎨 Scene resources loaded: {} textures for {} layers",
            renderer.texture_count(),
            renderer.get_visible_layers().len()
        );

        Ok(Self {
            renderer,
            last_frame_time: Instant::now(),
            target_fps: 60,
            output_info: output_info.clone(),
            initialized: false,
        })
    }

    /// Initialize OpenGL resources (must be called with GL context active)
    pub fn init_gl(&mut self, _egl_ctx: &EglContext) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        info!("🎨 Initializing scene OpenGL for {}", self.output_info.name);

        // Initialize GL resources
        self.renderer
            .init_gl()
            .context("Failed to initialize scene GL")?;
        self.initialized = true;

        Ok(())
    }

    /// Render a frame to the current EGL window
    pub fn render_frame(&mut self, egl_ctx: &EglContext, window: &EglWindow) -> Result<bool> {
        if !self.initialized {
            return Ok(false);
        }

        // Calculate delta time
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f64();
        self.last_frame_time = now;

        // Update animations
        self.renderer.update(delta);

        // Make context current
        egl_ctx.make_current(window)?;

        // Render scene
        self.renderer.render()?;

        // Swap buffers
        egl_ctx.swap_buffers(window)?;

        Ok(true)
    }

    /// Get target frame time in milliseconds
    pub fn frame_time_ms(&self) -> u64 {
        1000 / self.target_fps as u64
    }

    /// Set target FPS
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps.clamp(1, 144);
    }

    /// Get scene resolution
    pub fn resolution(&self) -> (u32, u32) {
        self.renderer.resolution()
    }

    /// Get number of loaded textures
    pub fn texture_count(&self) -> usize {
        self.renderer.texture_count()
    }

    /// Check if scene has renderable content
    pub fn has_content(&self) -> bool {
        self.renderer.has_content()
    }

    /// Clean up resources
    pub fn cleanup(&mut self) {
        if self.initialized {
            debug!("Cleaning up scene player for {}", self.output_info.name);
            self.renderer.cleanup_gl();
            self.initialized = false;
        }
    }
}

impl Drop for ScenePlayer {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OutputHdrCapabilities;

    #[test]
    fn test_scene_player_creation() {
        // This test requires a real scene directory
        let home = std::env::var("HOME").unwrap_or_default();
        let scene_dir = std::path::PathBuf::from(&home)
            .join(".steam/steam/steamapps/workshop/content/431960/3578699777");

        if !scene_dir.exists() {
            println!("Skipping test: scene directory not found");
            return;
        }

        let output_info = OutputInfo {
            name: "test".to_string(),
            width: 1920,
            height: 1080,
            scale: 1.0,
            position: (0, 0),
            active: true,
            hdr_capabilities: OutputHdrCapabilities::default(),
        };

        let player = ScenePlayer::new(&scene_dir, &output_info);
        assert!(
            player.is_ok(),
            "Should create scene player: {:?}",
            player.err()
        );

        let player = player.unwrap();
        assert_eq!(player.resolution(), (1920, 1080));

        // Verify textures were loaded from PKG
        let texture_count = player.texture_count();
        println!("✅ Scene player created:");
        println!("   Resolution: {:?}", player.resolution());
        println!("   Textures: {}", texture_count);
        println!("   Has content: {}", player.has_content());

        assert!(texture_count > 0, "Should have loaded textures from PKG");
        assert!(player.has_content(), "Should have renderable content");
    }
}
