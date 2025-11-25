use anyhow::{anyhow, Result};
use std::ffi::{c_char, c_void, CString};
use std::ptr;
use tracing::{debug, info, warn};

use crate::config::EffectiveConfig;
use crate::core::types::{HwdecMode, OutputInfo};
use crate::video::egl::EglContext;

// mpv_render_param_type constants (from libmpv/render.h)
const MPV_RENDER_PARAM_INVALID: u32 = 0;
const MPV_RENDER_PARAM_API_TYPE: u32 = 1;
const MPV_RENDER_PARAM_OPENGL_INIT_PARAMS: u32 = 2;
const MPV_RENDER_PARAM_OPENGL_FBO: u32 = 3;
const MPV_RENDER_PARAM_FLIP_Y: u32 = 4;

// OpenGL get_proc_address callback wrapper
extern "C" fn get_proc_address_wrapper(ctx: *mut c_void, name: *const c_char) -> *mut c_void {
    if ctx.is_null() || name.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let egl_ctx = &*(ctx as *const EglContext);
        let name_str = std::ffi::CStr::from_ptr(name).to_str().unwrap_or("");
        egl_ctx.get_proc_address(name_str) as *mut c_void
    }
}

/// MPV-based video player using direct libmpv-sys FFI with OpenGL rendering
pub struct MpvPlayer {
    handle: *mut libmpv_sys::mpv_handle,
    render_context: Option<*mut libmpv_sys::mpv_render_context>,
    output_info: OutputInfo,
    // Cache video dimensions to avoid repeated property access
    cached_dimensions: Option<(i32, i32)>,
}

// Safety: mpv_handle can be safely sent between threads
unsafe impl Send for MpvPlayer {}

impl MpvPlayer {
    pub fn new(config: &EffectiveConfig, output_info: &OutputInfo) -> Result<Self> {
        info!("🎬 Initializing libmpv for output {}", output_info.name);

        // Create MPV instance using raw FFI
        let handle = unsafe { libmpv_sys::mpv_create() };
        if handle.is_null() {
            return Err(anyhow!("Failed to create MPV handle"));
        }

        // Helper to set string option
        let set_option = |name: &str, value: &str| {
            let name_c = CString::new(name).unwrap();
            let value_c = CString::new(value).unwrap();
            unsafe {
                let ret =
                    libmpv_sys::mpv_set_option_string(handle, name_c.as_ptr(), value_c.as_ptr());
                if ret < 0 {
                    warn!("Failed to set option {}={}: error {}", name, value, ret);
                }
            }
        };

        // Configure MPV
        set_option("config", "no");
        set_option("terminal", "no");
        set_option("msg-level", "all=warn");

        // Video output - use libmpv for render API
        set_option("vo", "libmpv");
        set_option("vid", "auto");

        // Configure scaling behavior based on layout mode
        match config.layout {
            crate::core::types::LayoutMode::Fill | crate::core::types::LayoutMode::Cover => {
                // Fill screen, crop if needed (like Wallpaper Engine Fill mode)
                set_option("keepaspect", "yes");
                set_option("panscan", "1.0"); // Scale to cover, crop edges
                set_option("video-align-x", "0"); // Center horizontal
                set_option("video-align-y", "0"); // Center vertical
            }
            crate::core::types::LayoutMode::Stretch => {
                // Stretch to fill, ignore aspect ratio (like WE Stretch mode)
                set_option("keepaspect", "no");
                set_option("video-unscaled", "no");
            }
            crate::core::types::LayoutMode::Contain => {
                // Fit inside screen, maintain aspect (like WE Fit mode)
                set_option("keepaspect", "yes");
                set_option("panscan", "0.0"); // No cropping
                set_option("video-align-x", "0"); // Center horizontal
                set_option("video-align-y", "0"); // Center vertical
            }
            crate::core::types::LayoutMode::Centre => {
                // Center without scaling (like WE Center mode)
                set_option("keepaspect", "yes");
                set_option("video-unscaled", "yes");
                set_option("video-align-x", "0");
                set_option("video-align-y", "0");
            }
        }

        // Memory optimization: Limit video output queue
        set_option("video-latency-hacks", "yes"); // Reduce buffering
        set_option("vd-lavc-dr", "yes"); // Enable direct rendering (less copies)
        set_option("opengl-swapinterval", "1"); // Sync with display refresh;

        // Playback settings
        if config.r#loop {
            set_option("loop-file", "inf");
        }

        // Hardware decoding
        let hwdec_mode: HwdecMode = config.hwdec.into();
        let hwdec_str = match hwdec_mode {
            HwdecMode::Auto => "auto-safe",
            HwdecMode::Force => "yes",
            HwdecMode::No => "no",
        };
        set_option("hwdec", hwdec_str);

        // Audio settings
        if config.mute {
            set_option("mute", "yes");
        } else {
            let volume = format!("{}", (config.volume * 100.0) as i64);
            set_option("volume", &volume);
        }

        // Start time
        if config.start_time > 0.0 {
            let start = format!("{}", config.start_time);
            set_option("start", &start);
        }

        // Playback rate
        if (config.playback_rate - 1.0).abs() > 0.01 {
            let speed = format!("{}", config.playback_rate);
            set_option("speed", &speed);
        }

        // Initialize MPV
        let ret = unsafe { libmpv_sys::mpv_initialize(handle) };
        if ret < 0 {
            unsafe { libmpv_sys::mpv_terminate_destroy(handle) };
            return Err(anyhow!("Failed to initialize MPV: error {}", ret));
        }

        info!("  ✓ MPV initialized successfully");

        // Memory optimization: Limit demuxer cache
        // Reduce memory footprint by limiting internal buffering
        set_option("demuxer-max-bytes", "50M"); // Limit demuxer cache to 50MB
        set_option("demuxer-max-back-bytes", "10M"); // Limit backward seek cache to 10MB

        // Configure source-specific options
        if config.source.is_streaming() {
            info!("  🌐 Configuring for streaming source");
            set_option("cache", "yes");
            set_option("cache-secs", "10");
            set_option("demuxer-max-bytes", "100M"); // Streaming needs more cache
        }

        if config.source.is_image_sequence() {
            info!("  �️  Configuring for image sequence");
            set_option("image-display-duration", "inf");
            // Get FPS for image sequences
            if let crate::core::types::VideoSource::ImageSequence { fps, .. } = &config.source {
                let fps_str = format!("{}", fps);
                set_option("fps", &fps_str);
            }
        }

        // Load video source
        let source_path = config.source.get_mpv_path();
        info!("  📁 Loading source: {}", source_path);

        let cmd = CString::new("loadfile").unwrap();
        let path_c = CString::new(source_path.as_str())?;
        let mode = CString::new("replace").unwrap();

        let mut args = [
            cmd.as_ptr(),
            path_c.as_ptr(),
            mode.as_ptr(),
            std::ptr::null(),
        ];

        let ret = unsafe { libmpv_sys::mpv_command(handle, args.as_mut_ptr()) };
        if ret < 0 {
            warn!("Failed to load source: error {}", ret);
        } else {
            info!("  ✓ Source loaded successfully");
        }

        Ok(Self {
            handle,
            render_context: None,
            output_info: output_info.clone(),
            cached_dimensions: None,
        })
    }

    /// Initialize OpenGL render context for video rendering
    pub fn init_render_context(&mut self, egl_context: &EglContext) -> Result<()> {
        if self.render_context.is_some() {
            return Ok(());
        }

        info!("🎨 Initializing mpv render context for OpenGL");

        // OpenGL initialization parameters
        let get_proc_address: extern "C" fn(*mut c_void, *const i8) -> *mut c_void =
            get_proc_address_wrapper;
        let get_proc_address_ctx = egl_context as *const _ as *mut c_void;

        let opengl_init_params = libmpv_sys::mpv_opengl_init_params {
            get_proc_address: Some(get_proc_address),
            get_proc_address_ctx,
            extra_exts: ptr::null(),
        };

        // Render API parameters
        let api_type = CString::new("opengl").unwrap();
        let params = [
            libmpv_sys::mpv_render_param {
                type_: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            libmpv_sys::mpv_render_param {
                type_: MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
                data: &opengl_init_params as *const _ as *mut c_void,
            },
            libmpv_sys::mpv_render_param {
                type_: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let mut render_context: *mut libmpv_sys::mpv_render_context = ptr::null_mut();
        let ret = unsafe {
            libmpv_sys::mpv_render_context_create(
                &mut render_context,
                self.handle,
                params.as_ptr() as *mut _,
            )
        };

        if ret < 0 {
            return Err(anyhow!(
                "Failed to create mpv render context: error {}",
                ret
            ));
        }

        self.render_context = Some(render_context);
        info!("  ✓ Render context created successfully");

        Ok(())
    }

    /// Configure HDR handling based on video content and user settings
    ///
    /// This method should be called after the video is loaded to:
    /// 1. Detect if the video contains HDR content
    /// 2. Apply appropriate tone mapping or HDR passthrough settings
    /// 3. Log HDR information for user visibility
    pub fn configure_hdr(&mut self, config: &EffectiveConfig) -> Result<()> {
        use crate::video::hdr::HdrMode;

        info!("🎨 Configuring HDR handling...");

        // Check user's HDR mode preference
        match config.hdr_mode {
            HdrMode::Disable => {
                info!("  HDR mode: Disabled (forced SDR)");
                // No special configuration needed, MPV defaults to SDR
                return Ok(());
            }
            HdrMode::Force => {
                info!("  HDR mode: Force (always apply HDR processing)");
                // Force HDR tone mapping even for SDR content
                self.configure_tone_mapping(config)?;
                return Ok(());
            }
            HdrMode::Auto => {
                // Continue with auto-detection
                info!("  HDR mode: Auto (detect from video)");
            }
        }

        // Try to detect HDR metadata from the video
        // Note: This might return None if video hasn't started playing yet
        match self.get_hdr_metadata() {
            Some(metadata) => {
                info!("  📊 Video HDR metadata detected:");
                info!("    Color space: {:?}", metadata.color_space);
                info!("    Transfer function: {:?}", metadata.transfer_function);
                info!("    Primaries: {}", metadata.primaries);
                if let Some(peak) = metadata.peak_luminance {
                    info!("    Peak luminance: {:.1} nits", peak);
                }

                if metadata.is_hdr() {
                    info!(
                        "  ✨ HDR content detected: {}",
                        metadata.format_description()
                    );

                    // Check if output supports HDR (currently always false)
                    if self.output_info.hdr_capabilities.hdr_supported {
                        info!("  🖥️  Output supports HDR - enabling passthrough");
                        self.configure_hdr_passthrough()?;
                    } else {
                        info!("  🖥️  Output is SDR - enabling tone mapping");
                        self.configure_tone_mapping(config)?;
                    }
                } else {
                    info!("  📺 SDR content detected - no HDR processing needed");
                }
            }
            None => {
                debug!("  ⚠️  Could not detect HDR metadata (video may not be loaded yet)");
                debug!("    Will use default settings");
                // For now, configure tone mapping as a safe default
                // It won't hurt SDR content and will help if HDR is loaded later
                self.configure_tone_mapping(config)?;
            }
        }

        Ok(())
    }

    /// Configure tone mapping for HDR to SDR conversion
    fn configure_tone_mapping(&self, config: &EffectiveConfig) -> Result<()> {
        use crate::video::hdr::ContentType;

        info!("  🎨 Configuring tone mapping for HDR → SDR");

        // Clone config to allow modifications for content-aware optimization
        let mut optimized_config = config.tone_mapping.clone();

        // Apply content-aware optimizations if HDR metadata is available
        if let Some(metadata) = self.get_hdr_metadata() {
            let content_type = ContentType::detect_from_metadata(&metadata);
            debug!("    Content type: {:?}", content_type);

            optimized_config.optimize_for_content(&metadata);

            if optimized_config.param != config.tone_mapping.param {
                info!(
                    "    📊 Applied content-aware param optimization: {:.2}",
                    optimized_config.param
                );
            }
            if optimized_config.mode != config.tone_mapping.mode {
                info!(
                    "    📊 Applied content-aware mode optimization: {}",
                    optimized_config.mode
                );
            }
        }

        let set_option = |name: &str, value: &str| {
            let name_c = CString::new(name).unwrap();
            let value_c = CString::new(value).unwrap();
            unsafe {
                let ret = libmpv_sys::mpv_set_option_string(
                    self.handle,
                    name_c.as_ptr(),
                    value_c.as_ptr(),
                );
                if ret < 0 {
                    warn!("    Failed to set {}={}: error {}", name, value, ret);
                } else {
                    debug!("    ✓ Set {}={}", name, value);
                }
            }
        };

        // Set tone mapping algorithm
        let algorithm = optimized_config.algorithm.as_mpv_str();
        set_option("tone-mapping", algorithm);
        info!(
            "    Algorithm: {} ({})",
            algorithm,
            optimized_config.algorithm.description()
        );

        // Set tone mapping mode
        set_option("tone-mapping-mode", &optimized_config.mode);
        info!("    Mode: {}", optimized_config.mode);

        // Enable/disable dynamic peak detection
        if optimized_config.compute_peak {
            set_option("hdr-compute-peak", "yes");
            info!("    Dynamic peak detection: enabled");
        } else {
            set_option("hdr-compute-peak", "no");
        }

        // Set tone mapping parameter if algorithm uses it
        if optimized_config.algorithm.uses_param() {
            let param = format!("{:.2}", optimized_config.param);
            set_option("tone-mapping-param", &param);
            info!("    Parameter: {}", param);
        }

        // Set target color space for SDR
        set_option("target-trc", "srgb");
        set_option("target-prim", "bt.709");
        set_option("target-peak", "203"); // Typical SDR peak brightness
        debug!("    Target: sRGB/BT.709 @ 203 nits");

        info!("  ✓ Tone mapping configured");
        Ok(())
    }

    /// Configure HDR passthrough (for future use when output supports HDR)
    fn configure_hdr_passthrough(&self) -> Result<()> {
        info!("  🎨 Configuring HDR passthrough");

        let set_option = |name: &str, value: &str| {
            let name_c = CString::new(name).unwrap();
            let value_c = CString::new(value).unwrap();
            unsafe {
                let ret = libmpv_sys::mpv_set_option_string(
                    self.handle,
                    name_c.as_ptr(),
                    value_c.as_ptr(),
                );
                if ret < 0 {
                    warn!("    Failed to set {}={}: error {}", name, value, ret);
                } else {
                    debug!("    ✓ Set {}={}", name, value);
                }
            }
        };

        // Enable HDR passthrough
        set_option("target-colorspace-hint", "yes");
        set_option("icc-profile-auto", "yes");

        // Disable tone mapping for passthrough
        set_option("tone-mapping", "clip");

        info!("  ✓ HDR passthrough configured");
        Ok(())
    }

    /// Render a video frame to the current OpenGL context
    pub fn render(&mut self, width: i32, height: i32, fbo: i32) -> Result<()> {
        let Some(render_ctx) = self.render_context else {
            debug!("No render context available");
            return Ok(());
        };

        debug!("🎬 Rendering frame: {}x{} to FBO {}", width, height, fbo);

        // FBO parameters
        let fbo_data = libmpv_sys::mpv_opengl_fbo {
            fbo,
            w: width,
            h: height,
            internal_format: 0, // 0 = auto
        };

        let flip_y: i32 = 1;

        let params = [
            libmpv_sys::mpv_render_param {
                type_: MPV_RENDER_PARAM_OPENGL_FBO,
                data: &fbo_data as *const _ as *mut c_void,
            },
            libmpv_sys::mpv_render_param {
                type_: MPV_RENDER_PARAM_FLIP_Y,
                data: &flip_y as *const _ as *mut c_void,
            },
            libmpv_sys::mpv_render_param {
                type_: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let ret =
            unsafe { libmpv_sys::mpv_render_context_render(render_ctx, params.as_ptr() as *mut _) };

        if ret < 0 {
            warn!("mpv render error: {}", ret);
        } else {
            debug!("  ✓ Frame rendered successfully");
        }

        Ok(())
    }

    /// Pause playback (for future power management)
    #[allow(dead_code)]
    pub fn pause(&mut self) -> Result<()> {
        let prop = CString::new("pause").unwrap();
        let value = CString::new("yes").unwrap();
        let ret = unsafe {
            libmpv_sys::mpv_set_option_string(self.handle, prop.as_ptr(), value.as_ptr())
        };
        if ret < 0 {
            return Err(anyhow!("Failed to pause: error {}", ret));
        }
        Ok(())
    }

    /// Resume playback (for future power management)
    #[allow(dead_code)]
    pub fn resume(&mut self) -> Result<()> {
        let prop = CString::new("pause").unwrap();
        let value = CString::new("no").unwrap();
        let ret = unsafe {
            libmpv_sys::mpv_set_option_string(self.handle, prop.as_ptr(), value.as_ptr())
        };
        if ret < 0 {
            return Err(anyhow!("Failed to resume: error {}", ret));
        }
        Ok(())
    }

    /// Get video dimensions (width, height)
    /// Returns None if video is not loaded or dimensions are not available
    /// Caches result to avoid repeated property access
    #[inline]
    pub fn get_video_dimensions(&mut self) -> Option<(i32, i32)> {
        // Return cached value if available (fast path)
        if let Some(dims) = self.cached_dimensions {
            return Some(dims);
        }

        // Query MPV for dimensions
        let width = self.get_property_i64("dwidth")?;
        let height = self.get_property_i64("dheight")?;

        if width > 0 && height > 0 {
            let dims = (width as i32, height as i32);
            // Cache for future calls
            self.cached_dimensions = Some(dims);
            Some(dims)
        } else {
            None
        }
    }

    /// Invalidate cached dimensions (call when video changes)
    #[allow(dead_code)]
    pub fn invalidate_dimensions_cache(&mut self) {
        self.cached_dimensions = None;
    }

    /// Get an i64 property from MPV
    #[inline]
    fn get_property_i64(&self, name: &str) -> Option<i64> {
        let prop_name = CString::new(name).ok()?;
        let mut value: i64 = 0;

        let ret = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                prop_name.as_ptr(),
                4, // MPV_FORMAT_INT64
                &mut value as *mut i64 as *mut c_void,
            )
        };

        (ret == 0).then_some(value)
    }

    /// Get a string property from MPV
    fn get_property_string(&self, name: &str) -> Option<String> {
        let prop_name = CString::new(name).ok()?;

        let ret = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                prop_name.as_ptr(),
                1, // MPV_FORMAT_STRING
                std::ptr::null_mut(),
            )
        };

        if ret != 0 {
            return None;
        }

        let mut value_ptr: *mut c_char = std::ptr::null_mut();
        let ret = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                prop_name.as_ptr(),
                1, // MPV_FORMAT_STRING
                &mut value_ptr as *mut *mut c_char as *mut c_void,
            )
        };

        if ret == 0 && !value_ptr.is_null() {
            let c_str = unsafe { std::ffi::CStr::from_ptr(value_ptr) };
            let result = c_str.to_string_lossy().into_owned();
            unsafe {
                libmpv_sys::mpv_free(value_ptr as *mut c_void);
            }
            Some(result)
        } else {
            None
        }
    }

    /// Get a f64 property from MPV
    #[inline]
    fn get_property_f64(&self, name: &str) -> Option<f64> {
        let prop_name = CString::new(name).ok()?;
        let mut value: f64 = 0.0;

        let ret = unsafe {
            libmpv_sys::mpv_get_property(
                self.handle,
                prop_name.as_ptr(),
                5, // MPV_FORMAT_DOUBLE
                &mut value as *mut f64 as *mut c_void,
            )
        };

        (ret == 0).then_some(value)
    }

    /// Get HDR metadata from the currently playing video
    pub fn get_hdr_metadata(&self) -> Option<crate::video::hdr::HdrMetadata> {
        use crate::video::hdr::{parse_colorspace, parse_transfer_function, HdrMetadata};

        // Query color space properties
        let colorspace_str = self.get_property_string("video-params/colorspace")?;
        let gamma_str = self.get_property_string("video-params/gamma")?;
        let primaries_str = self.get_property_string("video-params/primaries")?;

        // Parse color space and transfer function
        let color_space = parse_colorspace(&colorspace_str);
        let transfer_function = parse_transfer_function(&gamma_str);

        // Query peak luminance (sig-peak)
        let peak_luminance = self.get_property_f64("video-params/sig-peak");

        Some(HdrMetadata {
            color_space,
            transfer_function,
            primaries: primaries_str,
            peak_luminance,
            avg_luminance: None, // Not directly available from MPV
            min_luminance: None, // Not directly available from MPV
        })
    }
}

impl MpvPlayer {
    /// Seek to specific time (in seconds)
    /// TODO: Re-enable in v0.5.0 with per-surface state management
    #[allow(dead_code)]
    pub fn seek(&mut self, time: f64) -> Result<()> {
        let cmd = format!("seek {} absolute", time);
        self.command(&cmd)
    }

    /// Load a new video file
    /// TODO: Re-enable in v0.5.0 with per-surface state management
    #[allow(dead_code)]
    pub fn load_file(&mut self, path: &str) -> Result<()> {
        let cmd = format!("loadfile {}", path);
        self.invalidate_dimensions_cache();
        self.command(&cmd)
    }

    /// Set playback rate (speed)
    /// TODO: Re-enable in v0.5.0 with per-surface state management
    #[allow(dead_code)]
    pub fn set_playback_rate(&mut self, rate: f64) -> Result<()> {
        let cmd = format!("set speed {}", rate);
        self.command(&cmd)
    }

    /// Set volume (0.0 - 1.0)
    /// TODO: Re-enable in v0.5.0 with per-surface state management
    #[allow(dead_code)]
    pub fn set_volume(&mut self, volume: f64) -> Result<()> {
        let vol = volume * 100.0; // MPV uses 0-100 scale
        let cmd = format!("set volume {}", vol);
        self.command(&cmd)
    }

    /// Toggle mute
    /// TODO: Re-enable in v0.5.0 with per-surface state management
    #[allow(dead_code)]
    pub fn toggle_mute(&mut self) -> Result<()> {
        self.command("cycle mute")
    }

    /// Execute MPV command
    #[allow(dead_code)]
    fn command(&mut self, cmd: &str) -> Result<()> {
        let c_cmd = CString::new(cmd).unwrap();
        let mut args: Vec<*const i8> = vec![c_cmd.as_ptr(), std::ptr::null()];
        unsafe {
            let ret = libmpv_sys::mpv_command(self.handle, args.as_mut_ptr());
            if ret < 0 {
                return Err(anyhow!("MPV command '{}' failed: error {}", cmd, ret));
            }
        }
        Ok(())
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        debug!("Dropping MPV player for {}", self.output_info.name);

        // Free render context first
        if let Some(render_ctx) = self.render_context {
            unsafe {
                libmpv_sys::mpv_render_context_free(render_ctx);
            }
        }

        // Then terminate MPV handle
        if !self.handle.is_null() {
            unsafe {
                libmpv_sys::mpv_terminate_destroy(self.handle);
            }
        }
    }
}
