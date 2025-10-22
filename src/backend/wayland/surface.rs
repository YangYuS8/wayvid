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
use crate::video::mpv::MpvPlayer;

pub struct WaylandSurface {
    pub wl_surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub output_id: u32,
    pub output_info: OutputInfo,
    pub config: EffectiveConfig,

    // EGL/OpenGL rendering
    egl_window: Option<EglWindow>,
    gl_loaded: bool,

    // Video player
    #[cfg(feature = "video-mpv")]
    player: Option<MpvPlayer>,

    // Frame synchronization
    frame_callback: Option<wl_callback::WlCallback>,
    frame_pending: bool,

    // Layout cache (video_w, video_h, output_w, output_h) -> viewport
    cached_layout: Option<((i32, i32, i32, i32), (i32, i32, i32, i32))>,

    configured: bool,
    initial_configure_done: bool,
}

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

        // Configure layer surface for input passthrough
        layer_surface.set_exclusive_zone(0);
        layer_surface
            .set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);

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
            player: None,
            frame_callback: None,
            frame_pending: false,
            cached_layout: None,
            configured: false,
            initial_configure_done: false,
        })
    }

    pub fn configure(
        &mut self,
        width: u32,
        height: u32,
        serial: u32,
        egl_context: Option<&EglContext>,
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

        // Initialize EGL window on first configure
        if is_first && self.egl_window.is_none() {
            if let Some(egl_ctx) = egl_context {
                match egl_ctx.create_window(&self.wl_surface, width as i32, height as i32) {
                    Ok(egl_win) => {
                        self.egl_window = Some(egl_win);
                        info!(
                            "  ✓ EGL window created for output {}",
                            self.output_info.name
                        );
                    }
                    Err(e) => {
                        error!("Failed to create EGL window: {}", e);
                    }
                }
            }
        }

        // Resize EGL window if dimensions changed
        if let Some(ref mut egl_win) = self.egl_window {
            if let Err(e) = egl_win.resize(width as i32, height as i32) {
                error!("Failed to resize EGL window: {}", e);
            }
        }

        // Initialize player after first configuration
        #[cfg(feature = "video-mpv")]
        {
            if is_first && self.player.is_none() {
                match self.init_player(egl_context) {
                    Ok(()) => info!("✓ MPV player initialized for {}", self.output_info.name),
                    Err(e) => error!("Failed to initialize player: {}", e),
                }
            }
        }

        self.layer_surface.ack_configure(serial);

        // Only commit on initial configure to avoid loops
        if is_first {
            self.wl_surface.commit();
        }
    }

    #[cfg(feature = "video-mpv")]
    fn init_player(&mut self, egl_context: Option<&EglContext>) -> Result<()> {
        let mut player = MpvPlayer::new(&self.config, &self.output_info)?;

        // Initialize render context if EGL is available
        if let (Some(egl_ctx), Some(ref egl_win)) = (egl_context, &self.egl_window) {
            // Make OpenGL context current before initializing render context
            if let Err(e) = egl_ctx.make_current(egl_win) {
                error!("Failed to make context current: {}", e);
            } else if let Err(e) = player.init_render_context(egl_ctx) {
                error!("Failed to init render context: {}", e);
            } else {
                info!("  ✓ Render context initialized");
            }
        }

        self.player = Some(player);
        Ok(())
    }

    pub fn render(&mut self, egl_context: Option<&EglContext>) -> Result<()> {
        if !self.configured || !self.frame_pending {
            return Ok(());
        }

        // Clear frame pending flag
        self.frame_pending = false;

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
                if let Some(ref mut player) = self.player {
                    let output_w = egl_win.width();
                    let output_h = egl_win.height();
                    
                    // Get video dimensions and calculate/use cached layout
                    let (render_w, render_h) = if let Some((vw, vh)) = player.get_video_dimensions()
                    {
                        let cache_key = (vw, vh, output_w, output_h);
                        
                        // Check if we can use cached layout
                        let viewport = if let Some((cached_key, cached_viewport)) = &self.cached_layout {
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

                    // Render video frame
                    if let Err(e) = player.render(render_w, render_h, 0) {
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

    /// Destroy surface resources (for future hot-plug support)
    #[allow(dead_code)]
    pub fn destroy(&mut self) {
        self.layer_surface.destroy();
        self.wl_surface.destroy();
        info!("Destroyed surface for output {}", self.output_info.name);
    }
}
