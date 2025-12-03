use serde::{Deserialize, Serialize};

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

    /// Wallpaper Engine video project
    #[serde(rename = "WeProject")]
    WeProject { path: String },

    /// Wallpaper Engine scene project (JSON-based with layers)
    #[serde(rename = "WeScene")]
    WeScene { path: String },
}

// Manual Eq implementation for VideoSource (treating f64 as bits)
impl Eq for VideoSource {}

// Manual Hash implementation for VideoSource
impl std::hash::Hash for VideoSource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            VideoSource::File { path } => {
                0u8.hash(state);
                path.hash(state);
            }
            VideoSource::Directory { path } => {
                1u8.hash(state);
                path.hash(state);
            }
            VideoSource::Url { url } => {
                2u8.hash(state);
                url.hash(state);
            }
            VideoSource::Rtsp { url } => {
                3u8.hash(state);
                url.hash(state);
            }
            VideoSource::Pipe { path } => {
                4u8.hash(state);
                path.hash(state);
            }
            VideoSource::ImageSequence { path, fps } => {
                5u8.hash(state);
                path.hash(state);
                // Hash fps as bits to avoid f64 comparison issues
                fps.to_bits().hash(state);
            }
            VideoSource::WeProject { path } => {
                6u8.hash(state);
                path.hash(state);
            }
            VideoSource::WeScene { path } => {
                7u8.hash(state);
                path.hash(state);
            }
        }
    }
}

fn default_fps() -> f64 {
    30.0
}

impl VideoSource {
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
            VideoSource::WeScene { path } => {
                let expanded = shellexpand::tilde(path);
                expanded.to_string()
            }
        }
    }

    /// Get the source as a display string (for UI/logging)
    #[allow(dead_code)]
    #[inline]
    fn get_source_string(&self) -> &str {
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
            VideoSource::WeScene { path } => path,
        }
    }

    /// Check if this is a streaming source (needs special handling)
    #[inline]
    pub fn is_streaming(&self) -> bool {
        matches!(
            self,
            VideoSource::Url { .. } | VideoSource::Rtsp { .. } | VideoSource::Pipe { .. }
        )
    }

    /// Check if this is an image sequence (needs loop handling)
    #[inline]
    pub fn is_image_sequence(&self) -> bool {
        matches!(self, VideoSource::ImageSequence { .. })
    }

    /// Check if this is a scene wallpaper (needs scene renderer)
    #[inline]
    pub fn is_scene(&self) -> bool {
        matches!(self, VideoSource::WeScene { .. })
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

/// HDR capabilities of an output
#[derive(Debug, Clone)]
pub struct OutputHdrCapabilities {
    /// Whether the output supports HDR
    pub hdr_supported: bool,

    /// Maximum luminance in nits (if available)
    #[allow(dead_code)]
    pub max_luminance: Option<f64>,

    /// Minimum luminance in nits (if available)
    #[allow(dead_code)]
    pub min_luminance: Option<f64>,

    /// Supported transfer functions (EOTFs)
    #[allow(dead_code)]
    pub supported_eotf: Vec<crate::video::hdr::TransferFunction>,
}

impl Default for OutputHdrCapabilities {
    fn default() -> Self {
        Self {
            hdr_supported: false,
            max_luminance: Some(203.0), // Typical SDR peak brightness
            min_luminance: Some(0.0),
            supported_eotf: vec![crate::video::hdr::TransferFunction::Srgb],
        }
    }
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

    /// HDR capabilities of this output
    pub hdr_capabilities: OutputHdrCapabilities,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Render backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderBackend {
    /// Auto-detect best backend (prefer Vulkan if available)
    #[default]
    Auto,

    /// Force OpenGL (EGL) backend
    #[serde(rename = "opengl")]
    OpenGL,

    /// Force Vulkan backend
    Vulkan,
}

impl RenderBackend {
    /// Check if Vulkan backend is requested (explicit or auto)
    pub fn wants_vulkan(&self) -> bool {
        matches!(self, RenderBackend::Auto | RenderBackend::Vulkan)
    }

    /// Check if OpenGL backend is requested (explicit or auto)
    pub fn wants_opengl(&self) -> bool {
        matches!(self, RenderBackend::Auto | RenderBackend::OpenGL)
    }
}
