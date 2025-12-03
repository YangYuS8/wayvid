use anyhow::Result;
use std::ffi::CString;
use tracing::{error, info, warn};
use wayland_client::protocol::{wl_callback, wl_surface};
use wayland_client::QueueHandle;
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::config::EffectiveConfig;
use crate::core::layout::calculate_layout;
use crate::core::types::{OutputInfo, RenderBackend};
use crate::video::egl::{EglContext, EglWindow};
use crate::video::scene::ScenePlayer;

#[cfg(feature = "backend-vulkan")]
use crate::video::vulkan::VulkanWindow;

#[cfg(feature = "video-mpv")]
use crate::video::shared_decode::{DecoderHandle, SharedDecodeManager};

/// Active rendering backend for this surface
#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveBackend {
    /// OpenGL via EGL
    OpenGL,
    /// Vulkan (when backend-vulkan feature is enabled)
    #[cfg(feature = "backend-vulkan")]
    Vulkan,
}

/// Source rendering mode
#[derive(Debug, Clone, Copy, PartialEq)]
enum RenderMode {
    /// Video rendering via MPV
    Video,
    /// Scene rendering (Wallpaper Engine scene type)
    Scene,
}

pub struct WaylandSurface {
    pub wl_surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub output_id: u32,
    pub output_info: OutputInfo,
    pub config: EffectiveConfig,

    // Active rendering backend
    #[allow(dead_code)]
    active_backend: ActiveBackend,

    // EGL/OpenGL rendering
    egl_window: Option<EglWindow>,
    gl_loaded: bool,

    // Vulkan rendering (when feature enabled)
    #[cfg(feature = "backend-vulkan")]
    #[allow(dead_code)]
    vulkan_window: Option<VulkanWindow>,

    // Render mode (video or scene)
    render_mode: RenderMode,

    // Video decoder (shared across outputs with same source)
    #[cfg(feature = "video-mpv")]
    decoder_handle: Option<DecoderHandle>,

    // Scene player (for Wallpaper Engine scenes)
    scene_player: Option<ScenePlayer>,

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

        // Determine render mode based on source type
        let render_mode = if config.source.is_scene() {
            RenderMode::Scene
        } else {
            RenderMode::Video
        };

        // Determine active backend based on config
        // For now, always use OpenGL (Vulkan integration will be added later)
        let active_backend = Self::select_backend(&config.render_backend);

        info!(
            "  Backend: {:?} (requested: {:?})",
            active_backend, config.render_backend
        );

        Ok(Self {
            wl_surface,
            layer_surface,
            output_id,
            output_info,
            config,
            active_backend,
            egl_window: None,
            gl_loaded: false,
            #[cfg(feature = "backend-vulkan")]
            vulkan_window: None,
            render_mode,
            #[cfg(feature = "video-mpv")]
            decoder_handle: None,
            scene_player: None,
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

    /// Select the active rendering backend based on configuration
    fn select_backend(requested: &RenderBackend) -> ActiveBackend {
        match requested {
            RenderBackend::OpenGL => ActiveBackend::OpenGL,
            #[cfg(feature = "backend-vulkan")]
            RenderBackend::Vulkan => ActiveBackend::Vulkan,
            #[cfg(not(feature = "backend-vulkan"))]
            RenderBackend::Vulkan => {
                warn!("Vulkan backend requested but not compiled in, falling back to OpenGL");
                ActiveBackend::OpenGL
            }
            RenderBackend::Auto => {
                // Auto mode: prefer Vulkan if available and working, fallback to OpenGL
                #[cfg(feature = "backend-vulkan")]
                {
                    // TODO: Implement Vulkan availability check
                    // For now, default to OpenGL until Vulkan is fully integrated
                    info!("Auto backend: using OpenGL (Vulkan integration pending)");
                    ActiveBackend::OpenGL
                }
                #[cfg(not(feature = "backend-vulkan"))]
                {
                    ActiveBackend::OpenGL
                }
            }
        }
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
            // Set frame_pending to allow initial render after configure
            self.frame_pending = true;
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

    /// Initialize scene player for Wallpaper Engine scenes
    fn init_scene(&mut self, egl_context: &EglContext) -> Result<()> {
        use crate::core::types::VideoSource;

        let scene_path = match &self.config.source {
            VideoSource::WeScene { path } => {
                let expanded = shellexpand::tilde(path);
                std::path::PathBuf::from(expanded.to_string())
            }
            _ => {
                return Err(anyhow::anyhow!("Source is not a scene"));
            }
        };

        info!("  🎭 Loading scene from: {}", scene_path.display());

        // Create scene player
        let mut player = ScenePlayer::new(&scene_path, &self.output_info)?;

        info!(
            "  📦 Scene loaded: {}x{}, {} textures",
            player.resolution().0,
            player.resolution().1,
            player.texture_count()
        );

        // Initialize OpenGL resources
        player.init_gl(egl_context)?;
        info!("  ✓ Scene OpenGL initialized");

        self.scene_player = Some(player);
        Ok(())
    }

    pub fn render(
        &mut self,
        egl_context: Option<&EglContext>,
        qh: &QueueHandle<crate::backend::wayland::app::AppState>,
    ) -> Result<()> {
        if !self.configured || !self.frame_pending {
            return Ok(());
        }

        // Clear frame_pending - we're going to render this frame
        // This ensures we wait for the next frame callback before rendering again
        self.frame_pending = false;

        // Lazy initialization: Initialize resources on first render if not already done
        if !self.resources_initialized && self.is_active {
            if let Some(egl_ctx) = egl_context {
                info!(
                    "🚀 Lazy initialization for output {} (first render, mode: {:?})",
                    self.output_info.name, self.render_mode
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

                // CRITICAL: Make EGL context current BEFORE initializing decoder/render context
                // Both mpv and scene renderer require a valid OpenGL context to be current
                if let Some(ref egl_win) = self.egl_window {
                    if let Err(e) = egl_ctx.make_current(egl_win) {
                        error!("Failed to make context current during lazy init: {}", e);
                        return Ok(());
                    }
                }

                // Load OpenGL functions early for scene rendering
                if !self.gl_loaded {
                    egl_ctx.load_gl_functions();
                    self.gl_loaded = true;
                    info!(
                        "  ✓ OpenGL functions loaded for output {}",
                        self.output_info.name
                    );
                }

                // Initialize based on render mode
                match self.render_mode {
                    RenderMode::Scene => {
                        // Initialize scene player
                        if self.scene_player.is_none() {
                            match self.init_scene(egl_ctx) {
                                Ok(()) => info!("  ✓ Scene player initialized lazily"),
                                Err(e) => {
                                    error!("Failed to initialize scene player: {}", e);
                                    return Ok(());
                                }
                            }
                        }
                    }
                    RenderMode::Video => {
                        // Initialize video decoder
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

        // frame_pending is cleared by the caller (app.rs run_event_loop)

        // Increment frame count
        self.frame_count += 1;

        // Check memory pressure every 600 frames (~10 seconds at 60fps)
        #[cfg(feature = "video-mpv")]
        if self.frame_count % 600 == 0 {
            if let Some(ref handle) = self.decoder_handle {
                handle.handle_memory_pressure();
            }
        }

        // OpenGL rendering
        // Track whether we actually rendered a frame
        let mut frame_rendered = false;

        if let (Some(egl_ctx), Some(ref egl_win)) = (egl_context, &self.egl_window) {
            // Make context current
            if let Err(e) = egl_ctx.make_current(egl_win) {
                error!("Failed to make EGL context current: {}", e);
                return Ok(());
            }

            // Load OpenGL functions on first render (may already be loaded during init)
            if !self.gl_loaded {
                egl_ctx.load_gl_functions();
                self.gl_loaded = true;
                info!(
                    "OpenGL functions loaded for output {}",
                    self.output_info.name
                );
            }

            // Render based on mode
            match self.render_mode {
                RenderMode::Scene => {
                    // Scene rendering
                    if let Some(ref mut player) = self.scene_player {
                        match player.render_frame(egl_ctx, egl_win) {
                            Ok(rendered) => {
                                frame_rendered = rendered;
                            }
                            Err(e) => {
                                warn!("Scene render error: {}", e);
                            }
                        }
                    }
                }
                RenderMode::Video => {
                    // Always clear to black first to ensure clean background
                    unsafe {
                        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                    }

                    // Render video frame with layout
                    #[cfg(feature = "video-mpv")]
                    {
                        if let Some(ref mut handle) = self.decoder_handle {
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
                            match handle.render(render_w, render_h, 0) {
                                Ok(rendered) => {
                                    frame_rendered = rendered;
                                }
                                Err(e) => {
                                    warn!("Video render error: {}", e);
                                }
                            }

                            // Reset viewport to full output
                            unsafe {
                                gl::Viewport(0, 0, output_w, output_h);
                            }
                        }
                    }
                }
            }

            // Only swap buffers if we actually rendered a new frame
            // This saves GPU work when video fps < display refresh rate
            // Note: Scene always renders, so frame_rendered is always true for scenes
            if frame_rendered {
                if let Err(e) = egl_ctx.swap_buffers(egl_win) {
                    error!("Failed to swap buffers: {}", e);
                }

                #[cfg(feature = "video-mpv")]
                if let Some(ref handle) = self.decoder_handle {
                    handle.report_swap();
                }
            }
        }

        // Request frame callback BEFORE commit
        // This is crucial: the callback is registered with wl_surface.frame()
        // and will be triggered after the compositor displays this commit
        self.request_frame(qh);

        // Always commit to Wayland
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

        // Cleanup scene player
        if let Some(ref mut player) = self.scene_player {
            player.cleanup();
            info!("  ✓ Scene player cleaned up");
        }
        self.scene_player = None;

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
    /// Re-acquires decoder with new source
    #[cfg(feature = "video-mpv")]
    pub fn switch_source(&mut self, source: &str, egl_context: Option<&EglContext>) -> Result<()> {
        use crate::core::types::VideoSource;

        info!(
            "🔄 Switching source for {} to: {}",
            self.output_info.name, source
        );

        // Parse source string to VideoSource
        let new_source = if source.starts_with("http://") || source.starts_with("https://") {
            VideoSource::Url {
                url: source.to_string(),
            }
        } else if source.starts_with("rtsp://") {
            VideoSource::Rtsp {
                url: source.to_string(),
            }
        } else {
            // Assume it's a file path
            let path = shellexpand::tilde(source).to_string();
            if std::path::Path::new(&path).is_dir() {
                VideoSource::Directory { path }
            } else {
                VideoSource::File { path }
            }
        };

        // Update config with new source
        self.config.source = new_source;

        // Release old decoder handle (drop it)
        self.decoder_handle = None;

        // Re-acquire decoder with new config
        self.init_decoder(egl_context)?;

        info!(
            "✅ Source switched for {} to: {}",
            self.output_info.name, source
        );

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
