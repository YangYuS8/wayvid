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

        // Load video file
        let video_path = config
            .source
            .primary_path()
            .map_err(|e| anyhow!("Failed to get video path: {}", e))?;

        info!("  📁 Loading video: {:?}", video_path);

        let cmd = CString::new("loadfile").unwrap();
        let path_str = video_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid video path"))?;
        let path_c = CString::new(path_str)?;
        let mode = CString::new("replace").unwrap();

        let mut args = [
            cmd.as_ptr(),
            path_c.as_ptr(),
            mode.as_ptr(),
            std::ptr::null(),
        ];

        let ret = unsafe { libmpv_sys::mpv_command(handle, args.as_mut_ptr()) };
        if ret < 0 {
            warn!("Failed to load video file: error {}", ret);
        } else {
            info!("  ✓ Video loaded successfully");
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
    pub fn get_video_dimensions(&mut self) -> Option<(i32, i32)> {
        // Return cached value if available
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

        if ret == 0 {
            Some(value)
        } else {
            None
        }
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
