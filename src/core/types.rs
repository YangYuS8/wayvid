use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Video source specification
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum VideoSource {
    /// Single video file
    File { path: String },

    /// Directory containing videos (future: playlist)
    Directory { path: String },

    /// Wallpaper Engine project (future: import)
    #[serde(rename = "WeProject")]
    WeProject { path: String },
}

impl VideoSource {
    /// Get the primary file path to play
    pub fn primary_path(&self) -> Result<PathBuf, &'static str> {
        match self {
            VideoSource::File { path } => {
                let expanded = shellexpand::tilde(path);
                Ok(PathBuf::from(expanded.as_ref()))
            }
            VideoSource::Directory { .. } => Err("Directory source not yet implemented"),
            VideoSource::WeProject { .. } => Err("WeProject import not yet implemented"),
        }
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

    /// Whether output is currently active
    pub active: bool,
}

impl OutputInfo {
    /// Get logical dimensions (physical / scale)
    pub fn logical_size(&self) -> (f64, f64) {
        (
            self.width as f64 / self.scale,
            self.height as f64 / self.scale,
        )
    }
}

/// Playback state
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
