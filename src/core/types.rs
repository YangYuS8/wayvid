use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Video source specification
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum VideoSource {
    /// Single video file
    File { path: String },

    /// Directory containing videos (playlist)
    Directory { path: String },

    /// HTTP/HTTPS URL stream
    Url { url: String },

    /// RTSP stream
    Rtsp { url: String },

    /// Pipe input (stdin or named pipe)
    Pipe {
        #[serde(default)]
        path: String, // Empty string means stdin
    },

    /// GIF or image sequence
    ImageSequence {
        path: String,
        #[serde(default = "default_fps")]
        fps: f64,
    },

    /// Wallpaper Engine project (future: import)
    #[serde(rename = "WeProject")]
    WeProject { path: String },
}

fn default_fps() -> f64 {
    30.0
}

impl VideoSource {
    /// Get the primary file path or URL to play
    pub fn primary_path(&self) -> Result<PathBuf, &'static str> {
        match self {
            VideoSource::File { path } => {
                let expanded = shellexpand::tilde(path);
                Ok(PathBuf::from(expanded.as_ref()))
            }
            VideoSource::Directory { .. } => Err("Directory source not yet implemented"),
            VideoSource::Url { .. } => Err("Use get_mpv_path() for URL sources"),
            VideoSource::Rtsp { .. } => Err("Use get_mpv_path() for RTSP sources"),
            VideoSource::Pipe { .. } => Err("Use get_mpv_path() for pipe sources"),
            VideoSource::ImageSequence { .. } => {
                let expanded = shellexpand::tilde(self.get_source_string());
                Ok(PathBuf::from(expanded.as_ref()))
            }
            VideoSource::WeProject { .. } => Err("WeProject import not yet implemented"),
        }
    }

    /// Get the source path/URL as string for MPV
    pub fn get_mpv_path(&self) -> String {
        match self {
            VideoSource::File { path } => {
                let expanded = shellexpand::tilde(path);
                expanded.to_string()
            }
            VideoSource::Directory { path } => {
                let expanded = shellexpand::tilde(path);
                expanded.to_string()
            }
            VideoSource::Url { url } => url.clone(),
            VideoSource::Rtsp { url } => url.clone(),
            VideoSource::Pipe { path } => {
                if path.is_empty() {
                    "fd://0".to_string() // stdin
                } else {
                    let expanded = shellexpand::tilde(path);
                    expanded.to_string()
                }
            }
            VideoSource::ImageSequence { path, .. } => {
                let expanded = shellexpand::tilde(path);
                expanded.to_string()
            }
            VideoSource::WeProject { path } => {
                let expanded = shellexpand::tilde(path);
                expanded.to_string()
            }
        }
    }

    /// Get the source as a display string
    pub fn get_source_string(&self) -> &str {
        match self {
            VideoSource::File { path } => path,
            VideoSource::Directory { path } => path,
            VideoSource::Url { url } => url,
            VideoSource::Rtsp { url } => url,
            VideoSource::Pipe { path } => {
                if path.is_empty() {
                    "stdin"
                } else {
                    path
                }
            }
            VideoSource::ImageSequence { path, .. } => path,
            VideoSource::WeProject { path } => path,
        }
    }

    /// Check if this is a streaming source (needs special handling)
    pub fn is_streaming(&self) -> bool {
        matches!(
            self,
            VideoSource::Url { .. } | VideoSource::Rtsp { .. } | VideoSource::Pipe { .. }
        )
    }

    /// Check if this is an image sequence (needs loop handling)
    pub fn is_image_sequence(&self) -> bool {
        matches!(self, VideoSource::ImageSequence { .. })
    }
}

/// Layout/scaling mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum LayoutMode {
    /// Scale and crop to fill entire output (maintains aspect ratio)
    Fill,

    /// Scale to fit inside output (maintains aspect ratio, may have letterbox)
    Contain,

    /// Stretch to fill output (ignores aspect ratio)
    Stretch,

    /// Like Fill (alias for compatibility)
    Cover,

    /// Center without scaling
    Centre,
}

/// Output information
#[derive(Debug, Clone)]
pub struct OutputInfo {
    /// Output name (e.g., "HDMI-A-1", "eDP-1")
    pub name: String,

    /// Physical width in pixels
    pub width: i32,

    /// Physical height in pixels
    pub height: i32,

    /// Scale factor (1, 2, or fractional like 1.5)
    pub scale: f64,

    /// Logical position (x, y)
    pub position: (i32, i32),

    /// Whether output is currently active (for future hot-plug support)
    #[allow(dead_code)]
    pub active: bool,
}

impl OutputInfo {
    /// Get logical dimensions (physical / scale) (for future use)
    #[allow(dead_code)]
    pub fn logical_size(&self) -> (f64, f64) {
        (
            self.width as f64 / self.scale,
            self.height as f64 / self.scale,
        )
    }
}

/// Playback state (for future power management)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Error,
}

/// Hardware decode preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwdecMode {
    /// Try hardware decode, fallback to software
    Auto,

    /// Force hardware decode only
    #[allow(dead_code)]
    Force,

    /// Disable hardware decode
    No,
}

impl From<bool> for HwdecMode {
    fn from(enabled: bool) -> Self {
        if enabled {
            HwdecMode::Auto
        } else {
            HwdecMode::No
        }
    }
}
