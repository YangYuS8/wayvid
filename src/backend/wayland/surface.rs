use anyhow::Result;
use std::ffi::CString;
use tracing::{error, info, warn};
use wayland_client::protocol::{wl_callback, wl_surface};
use wayland_client::QueueHandle;
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::config::EffectiveConfig;
use crate::core::layout::calculate_layout;
use crate::core::types::OutputInfo;
use crate::video::egl::{EglContext, EglWindow};

#[cfg(feature = "video-mpv")]
use crate::video::shared_decode::{DecoderHandle, SharedDecodeManager};

pub struct WaylandSurface {
    pub wl_surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub output_id: u32,
    pub output_info: OutputInfo,
    pub config: EffectiveConfig,

    // EGL/OpenGL rendering
    egl_window: Option<EglWindow>,
    gl_loaded: bool,

    // Video decoder (shared across outputs with same source)
    #[cfg(feature = "video-mpv")]
    decoder_handle: Option<DecoderHandle>,

    // Frame synchronization
    frame_callback: Option<wl_callback::WlCallback>,
    frame_pending: bool,

    // Layout cache (video_w, video_h, output_w, output_h) -> viewport
    cached_layout: Option<LayoutCacheEntry>,

    configured: bool,
    initial_configure_done: bool,

    // Memory management
    frame_count: u64,

    // Lazy initialization state
    resources_initialized: bool,
    is_active: bool, // Whether this output is currently active/visible
}

// Type alias for complex layout cache type
type LayoutCacheEntry = ((i32, i32, i32, i32), (i32, i32, i32, i32));

impl WaylandSurface {
    pub fn new(
        wl_surface: wl_surface::WlSurface,
        layer_shell: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        output_id: u32,
        output_info: OutputInfo,
        config: EffectiveConfig,
        wl_output: &wayland_client::protocol::wl_output::WlOutput,
        qh: &QueueHandle<crate::backend::wayland::app::AppState>,
    ) -> Result<Self> {
        // Create layer surface
        let layer_surface = layer_shell.get_layer_surface(
            &wl_surface,
            Some(wl_output),
            zwlr_layer_shell_v1::Layer::Background,
            CString::new("wayvid").unwrap().into_string().unwrap(),
            qh,
            (),
        );

        // Configure layer surface for full screen coverage
        // -1 means ignore exclusive zones from other surfaces (panels, bars)
        // This ensures wallpaper covers entire screen including behind panels
        layer_surface.set_exclusive_zone(-1);
        layer_surface
            .set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);

        // Anchor to all edges for full coverage
        layer_surface.set_anchor(
            zwlr_layer_surface_v1::Anchor::Top
                | zwlr_layer_surface_v1::Anchor::Bottom
                | zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Right,
        );

        // Set size to output dimensions
        layer_surface.set_size(output_info.width as u32, output_info.height as u32);

        wl_surface.commit();

        info!(
            "Created layer surface for output {} ({}x{})",
            output_info.name, output_info.width, output_info.height
        );

        Ok(Self {
            wl_surface,
            layer_surface,
            output_id,
            output_info,
            config,
            egl_window: None,
            gl_loaded: false,
            #[cfg(feature = "video-mpv")]
            decoder_handle: None,
            frame_callback: None,
            frame_pending: false,
            cached_layout: None,
            configured: false,
            initial_configure_done: false,
            frame_count: 0,
            resources_initialized: false,
            is_active: true, // Assume active until proven otherwise
        })
    }

    pub fn configure(
        &mut self,
        width: u32,
        height: u32,
        serial: u32,
        _egl_context: Option<&EglContext>,
    ) {
        // Only log and process first configure
        let is_first = !self.initial_configure_done;

        if is_first {
            info!(
                "Initial configure for surface {} to {}x{}",
                self.output_info.name, width, height
            );
            self.initial_configure_done = true;
        }

        self.output_info.width = width as i32;
        self.output_info.height = height as i32;
        self.configured = true;

        // Resize EGL window if already initialized and dimensions changed
        if let Some(ref mut egl_win) = self.egl_window {
            if let Err(e) = egl_win.resize(width as i32, height as i32) {
                error!("Failed to resize EGL window: {}", e);
            }
        }

        // Note: Actual resource initialization is now deferred to first render
        // This is the key change for lazy initialization (Issue #15)

        self.layer_surface.ack_configure(serial);

        // Only commit on initial configure to avoid loops
        if is_first {
            self.wl_surface.commit();
        }
    }

    #[cfg(feature = "video-mpv")]
    fn init_decoder(&mut self, egl_context: Option<&EglContext>) -> Result<()> {
        // Acquire shared decoder from manager
        let manager = SharedDecodeManager::global();
        let handle =
            SharedDecodeManager::acquire_decoder(manager, &self.config, &self.output_info)?;

        info!(
            "  🔗 Decoder acquired for {} (source: {})",
            self.output_info.name,
            handle.source_description()
        );

        // Initialize render context if EGL is available
        if let (Some(egl_ctx), Some(ref egl_win)) = (egl_context, &self.egl_window) {
            // Make OpenGL context current before initializing render context
            if let Err(e) = egl_ctx.make_current(egl_win) {
                error!("Failed to make context current: {}", e);
            } else if let Err(e) = handle.init_render_context(egl_ctx) {
                error!("Failed to init render context: {}", e);
            } else {
                info!("  ✓ Render context initialized");
            }
        }

        self.decoder_handle = Some(handle);
        Ok(())
    }

    pub fn render(&mut self, egl_context: Option<&EglContext>) -> Result<()> {
        if !self.configured || !self.frame_pending {
            return Ok(());
        }

        // Lazy initialization: Initialize resources on first render if not already done
        if !self.resources_initialized && self.is_active {
            if let Some(egl_ctx) = egl_context {
                info!(
                    "🚀 Lazy initialization for output {} (first render)",
                    self.output_info.name
                );

                // Initialize EGL window
                if self.egl_window.is_none() {
                    match egl_ctx.create_window(
                        &self.wl_surface,
                        self.output_info.width,
                        self.output_info.height,
                    ) {
                        Ok(egl_win) => {
                            self.egl_window = Some(egl_win);
                            info!("  ✓ EGL window created lazily");
                        }
                        Err(e) => {
                            error!("Failed to create EGL window: {}", e);
                            return Ok(());
                        }
                    }
                }

                // Initialize decoder
                #[cfg(feature = "video-mpv")]
                if self.decoder_handle.is_none() {
                    match self.init_decoder(Some(egl_ctx)) {
                        Ok(()) => info!("  ✓ Decoder initialized lazily"),
                        Err(e) => {
                            error!("Failed to initialize decoder: {}", e);
                            return Ok(());
                        }
                    }
                }

                self.resources_initialized = true;
                info!(
                    "  ✅ Lazy initialization complete for {}",
                    self.output_info.name
                );
            } else {
                // No EGL context yet, skip rendering
                return Ok(());
            }
        }

        // Clear frame pending flag
        self.frame_pending = false;

        // Increment frame count
        self.frame_count += 1;

        // Check memory pressure every 600 frames (~10 seconds at 60fps)
        #[cfg(feature = "video-mpv")]
        if self.frame_count % 600 == 0 {
            if let Some(ref handle) = self.decoder_handle {
                handle.handle_memory_pressure();
            }
        }

        // OpenGL rendering with clear screen test
        if let (Some(egl_ctx), Some(ref egl_win)) = (egl_context, &self.egl_window) {
            // Make context current
            if let Err(e) = egl_ctx.make_current(egl_win) {
                error!("Failed to make EGL context current: {}", e);
                return Ok(());
            }

            // Load OpenGL functions on first render
            if !self.gl_loaded {
                egl_ctx.load_gl_functions();
                self.gl_loaded = true;
                info!(
                    "OpenGL functions loaded for output {}",
                    self.output_info.name
                );
            }

            // Clear to black background
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            // Render video frame with layout
            #[cfg(feature = "video-mpv")]
            {
                if let Some(ref handle) = self.decoder_handle {
                    let output_w = egl_win.width();
                    let output_h = egl_win.height();

                    // Get video dimensions and calculate/use cached layout
                    let (render_w, render_h) = if let Some((vw, vh)) = handle.dimensions() {
                        let cache_key = (vw, vh, output_w, output_h);

                        // Check if we can use cached layout
                        let viewport = if let Some((cached_key, cached_viewport)) =
                            &self.cached_layout
                        {
                            if cached_key == &cache_key {
                                *cached_viewport
                            } else {
                                // Cache miss - recalculate
                                let layout = calculate_layout(
                                    self.config.layout,
                                    vw,
                                    vh,
                                    output_w,
                                    output_h,
                                );
                                let viewport = layout.dst_rect;
                                self.cached_layout = Some((cache_key, viewport));
                                viewport
                            }
                        } else {
                            // First time - calculate and cache
                            let layout =
                                calculate_layout(self.config.layout, vw, vh, output_w, output_h);
                            let viewport = layout.dst_rect;
                            self.cached_layout = Some((cache_key, viewport));
                            viewport
                        };

                        // Set viewport to destination rectangle
                        let (x, y, w, h) = viewport;
                        unsafe {
                            gl::Viewport(x, y, w, h);
                        }

                        (w, h)
                    } else {
                        // No video dimensions yet, use full output
                        self.cached_layout = None;
                        (output_w, output_h)
                    };

                    // Render video frame via shared decoder
                    if let Err(e) = handle.render(render_w, render_h, 0) {
                        warn!("Video render error: {}", e);
                    }

                    // Reset viewport to full output
                    unsafe {
                        gl::Viewport(0, 0, output_w, output_h);
                    }
                }
            }

            // Swap buffers to display
            if let Err(e) = egl_ctx.swap_buffers(egl_win) {
                error!("Failed to swap buffers: {}", e);
            }
        }

        self.wl_surface.commit();
        Ok(())
    }

    /// Called when frame callback is triggered - marks surface ready for next render
    pub fn on_frame_ready(&mut self) {
        self.frame_pending = true;
    }

    /// Check if frame is pending for rendering
    pub fn has_frame_pending(&self) -> bool {
        self.frame_pending
    }

    /// Request next frame callback for vsync
    pub fn request_frame(&mut self, qh: &QueueHandle<crate::backend::wayland::app::AppState>) {
        // Old callback will be automatically destroyed when replaced
        // Request new frame callback with output_id as user data
        let callback = self.wl_surface.frame(qh, self.output_id);
        self.frame_callback = Some(callback);
    }

    /// Set output as active (visible/powered on)
    /// This enables lazy initialization on next render
    #[allow(dead_code)]
    pub fn set_active(&mut self, active: bool) {
        if self.is_active != active {
            self.is_active = active;
            if active {
                info!("Output {} marked as active", self.output_info.name);
            } else {
                info!("Output {} marked as inactive", self.output_info.name);
                // Optionally cleanup resources when inactive
                self.cleanup_resources();
            }
        }
    }

    /// Check if output is active
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Cleanup resources when output becomes inactive
    fn cleanup_resources(&mut self) {
        if !self.resources_initialized {
            return; // Nothing to cleanup
        }

        info!(
            "🧹 Cleaning up resources for inactive output {}",
            self.output_info.name
        );

        // Release decoder handle (decrements ref count)
        #[cfg(feature = "video-mpv")]
        {
            if self.decoder_handle.is_some() {
                self.decoder_handle = None;
                info!("  ✓ Decoder handle released");
            }
        }

        // Note: EGL window is kept for now as it's lightweight
        // and may be needed soon if output becomes active again

        self.resources_initialized = false;
    }

    /// Destroy surface resources (for future hot-plug support)
    #[allow(dead_code)]
    pub fn destroy(&mut self) {
        self.cleanup_resources();
        self.layer_surface.destroy();
        self.wl_surface.destroy();
        info!("Destroyed surface for output {}", self.output_info.name);
    }

    /// Pause video playback (for power management)
    /// TODO(M5): Shared decode architecture doesn't support per-surface playback control
    /// Need to design per-surface state management in future versions
    #[cfg(feature = "video-mpv")]
    #[allow(unused_variables)]
    pub fn pause_playback(&mut self) -> Result<()> {
        // Temporarily disabled - shared decoder affects all consumers
        warn!("pause_playback not supported with shared decode context");
        Ok(())
    }

    /// Resume video playback (for power management)
    /// TODO(M5): Shared decode architecture doesn't support per-surface playback control
    #[cfg(feature = "video-mpv")]
    #[allow(unused_variables)]
    pub fn resume_playback(&mut self) -> Result<()> {
        // Temporarily disabled - shared decoder affects all consumers
        warn!("resume_playback not supported with shared decode context");
        Ok(())
    }

    /// Get playback status
    /// TODO(M5): Implement per-surface status tracking
    #[cfg(feature = "video-mpv")]
    pub fn get_status(&self) -> Option<(bool, f64, f64)> {
        // Returns: (is_playing, current_time, duration)
        self.decoder_handle.as_ref()?;
        // For now return placeholder values
        // TODO: Add actual MPV property getters via decoder handle
        Some((true, 0.0, 0.0))
    }

    /// Seek to specific time
    /// TODO(M5): Shared decode architecture doesn't support per-surface seek
    #[cfg(feature = "video-mpv")]
    #[allow(unused_variables)]
    pub fn seek(&mut self, time: f64) -> Result<()> {
        // Temporarily disabled - seek affects all consumers
        warn!("seek not supported with shared decode context");
        Ok(())
    }

    /// Switch video source
    /// TODO(M5): Need to implement source switching with decoder re-acquisition
    #[cfg(feature = "video-mpv")]
    #[allow(unused_variables)]
    pub fn switch_source(&mut self, source: &str) -> Result<()> {
        // Temporarily disabled - requires decoder handle replacement
        warn!("switch_source not supported with shared decode context");
        Ok(())
    }

    /// Set playback rate
    /// TODO(M5): Shared decode architecture doesn't support per-surface rate control
    #[cfg(feature = "video-mpv")]
    #[allow(unused_variables)]
    pub fn set_playback_rate(&mut self, rate: f64) -> Result<()> {
        // Temporarily disabled - rate affects all consumers
        warn!("set_playback_rate not supported with shared decode context");
        Ok(())
    }

    /// Set volume
    /// TODO(M5): Audio handling needs to be designed for shared decode
    #[cfg(feature = "video-mpv")]
    #[allow(unused_variables)]
    pub fn set_volume(&mut self, volume: f64) -> Result<()> {
        // Temporarily disabled - audio is per-decoder not per-surface
        warn!("set_volume not supported with shared decode context");
        Ok(())
    }

    /// Toggle mute
    /// TODO(M5): Audio handling needs to be designed for shared decode
    #[cfg(feature = "video-mpv")]
    #[allow(unused_variables)]
    pub fn toggle_mute(&mut self) -> Result<()> {
        // Temporarily disabled - audio is per-decoder not per-surface
        warn!("toggle_mute not supported with shared decode context");
        Ok(())
    }

    /// Set layout mode
    pub fn set_layout(&mut self, layout: crate::core::types::LayoutMode) {
        self.config.layout = layout;
        // Invalidate layout cache
        self.cached_layout = None;
    }
}
