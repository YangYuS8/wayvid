use anyhow::{anyhow, Result};
use std::ffi::CString;
use tracing::{debug, info, warn};

use crate::config::EffectiveConfig;
use crate::core::types::{HwdecMode, OutputInfo};

/// MPV-based video player using direct libmpv-sys FFI
pub struct MpvPlayer {
    handle: *mut libmpv_sys::mpv_handle,
    output_info: OutputInfo,
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
                let ret = libmpv_sys::mpv_set_option_string(
                    handle,
                    name_c.as_ptr(),
                    value_c.as_ptr(),
                );
                if ret < 0 {
                    warn!("Failed to set option {}={}: error {}", name, value, ret);
                }
            }
        };

        // Configure MPV
        set_option("config", "no");
        set_option("terminal", "no");
        set_option("msg-level", "all=warn");

        // Video output - use null for now
        set_option("vo", "null");
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
        let path_str = video_path.to_str().ok_or_else(|| anyhow!("Invalid video path"))?;
        let path_c = CString::new(path_str)?;
        let mode = CString::new("replace").unwrap();

        let mut args = [cmd.as_ptr(), path_c.as_ptr(), mode.as_ptr(), std::ptr::null()];

        let ret = unsafe { libmpv_sys::mpv_command(handle, args.as_mut_ptr()) };
        if ret < 0 {
            warn!("Failed to load video file: error {}", ret);
        } else {
            info!("  ✓ Video loaded successfully");
        }

        Ok(Self {
            handle,
            output_info: output_info.clone(),
        })
    }

    pub fn render(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        let prop = CString::new("pause").unwrap();
        let value = CString::new("yes").unwrap();
        let ret = unsafe {
            libmpv_sys::mpv_set_option_string(
                self.handle,
                prop.as_ptr(),
                value.as_ptr(),
            )
        };
        if ret < 0 {
            return Err(anyhow!("Failed to pause: error {}", ret));
        }
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        let prop = CString::new("pause").unwrap();
        let value = CString::new("no").unwrap();
        let ret = unsafe {
            libmpv_sys::mpv_set_option_string(
                self.handle,
                prop.as_ptr(),
                value.as_ptr(),
            )
        };
        if ret < 0 {
            return Err(anyhow!("Failed to resume: error {}", ret));
        }
        Ok(())
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        debug!("Dropping MPV player for {}", self.output_info.name);
        if !self.handle.is_null() {
            unsafe {
                libmpv_sys::mpv_terminate_destroy(self.handle);
            }
        }
    }
}
