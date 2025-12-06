//! Engine commands, events, and configuration types

use std::collections::HashMap;
use std::path::PathBuf;

use wayvid_core::OutputInfo;

use crate::mpv::VideoConfig;

/// Commands sent from GUI to engine
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// Apply wallpaper to output(s)
    ApplyWallpaper {
        /// Path to video/image file
        path: PathBuf,
        /// Target output (None = all outputs)
        output: Option<String>,
    },

    /// Clear wallpaper from output(s)
    ClearWallpaper {
        /// Target output (None = all outputs)
        output: Option<String>,
    },

    /// Set volume for an output
    SetVolume {
        /// Target output
        output: String,
        /// Volume level (0.0 - 1.0)
        volume: f32,
    },

    /// Pause playback
    Pause {
        /// Target output (None = all outputs)
        output: Option<String>,
    },

    /// Resume playback
    Resume {
        /// Target output (None = all outputs)
        output: Option<String>,
    },

    /// Request current outputs list
    GetOutputs,

    /// Request current status
    GetStatus,

    /// Shutdown the engine
    Shutdown,
}

/// Events sent from engine to GUI
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// Engine has started successfully
    Started,

    /// Engine has stopped
    Stopped,

    /// New output detected
    OutputAdded(OutputInfo),

    /// Output removed
    OutputRemoved(String),

    /// Current outputs list (response to GetOutputs)
    OutputsList(Vec<OutputInfo>),

    /// Current status (response to GetStatus)
    Status(EngineStatus),

    /// Wallpaper applied successfully
    WallpaperApplied {
        /// Output name
        output: String,
        /// Wallpaper path
        path: PathBuf,
    },

    /// Wallpaper cleared
    WallpaperCleared {
        /// Output name
        output: String,
    },

    /// Error occurred
    Error(String),
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Video playback configuration
    pub video: VideoConfig,
    /// Auto-start playback when wallpaper is applied
    pub auto_play: bool,
    /// FPS limit (None = vsync/unlimited)
    pub fps_limit: Option<u32>,
    /// Pause playback when on battery power
    pub pause_on_battery: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            video: VideoConfig::default(),
            auto_play: true,
            fps_limit: None,
            pause_on_battery: false,
        }
    }
}

/// Current engine status
#[derive(Debug, Clone)]
pub struct EngineStatus {
    /// Whether engine is running
    pub running: bool,
    /// Available outputs
    pub outputs: Vec<OutputInfo>,
    /// Active wallpapers per output
    pub active_wallpapers: HashMap<String, Option<PathBuf>>,
}

impl Default for EngineStatus {
    fn default() -> Self {
        Self {
            running: false,
            outputs: Vec::new(),
            active_wallpapers: HashMap::new(),
        }
    }
}
