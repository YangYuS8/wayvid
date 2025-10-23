use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::types::{LayoutMode, VideoSource};

/// Global configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Video source
    pub source: VideoSource,

    /// Layout mode (Fill, Contain, Stretch, Cover, Centre)
    #[serde(default = "default_layout")]
    pub layout: LayoutMode,

    /// Loop playback
    #[serde(default = "default_loop")]
    pub r#loop: bool,

    /// Start time in seconds
    #[serde(default)]
    pub start_time: f64,

    /// Playback rate (speed)
    #[serde(default = "default_playback_rate")]
    pub playback_rate: f64,

    /// Mute audio
    #[serde(default = "default_mute")]
    pub mute: bool,

    /// Volume (0.0 - 1.0)
    #[serde(default)]
    pub volume: f64,

    /// Enable hardware decoding
    #[serde(default = "default_hwdec")]
    pub hwdec: bool,

    /// Per-output overrides (keyed by output name)
    #[serde(default)]
    pub per_output: HashMap<String, OutputConfig>,

    /// Power saving options
    #[serde(default)]
    pub power: PowerConfig,
}

/// Per-output configuration overrides
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<VideoSource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub playback_rate: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
}

/// Power saving configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PowerConfig {
    /// Pause when output is not visible
    #[serde(default = "default_pause_hidden")]
    pub pause_when_hidden: bool,

    /// Pause on battery
    #[serde(default)]
    pub pause_on_battery: bool,

    /// Target FPS limit (0 = unlimited)
    #[serde(default)]
    pub max_fps: u32,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            pause_when_hidden: true,
            pause_on_battery: false,
            max_fps: 0,
        }
    }
}

impl Config {
    /// Load configuration from YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML configuration")
    }

    /// Get effective configuration for a specific output
    pub fn for_output(&self, output_name: &str) -> EffectiveConfig {
        let base = self.clone();

        if let Some(override_cfg) = self.per_output.get(output_name) {
            EffectiveConfig {
                source: override_cfg.source.clone().unwrap_or(base.source),
                layout: override_cfg.layout.unwrap_or(base.layout),
                r#loop: base.r#loop,
                start_time: override_cfg.start_time.unwrap_or(base.start_time),
                playback_rate: override_cfg.playback_rate.unwrap_or(base.playback_rate),
                mute: override_cfg.mute.unwrap_or(base.mute),
                volume: override_cfg.volume.unwrap_or(base.volume),
                hwdec: base.hwdec,
                power: base.power.clone(),
            }
        } else {
            EffectiveConfig {
                source: base.source,
                layout: base.layout,
                r#loop: base.r#loop,
                start_time: base.start_time,
                playback_rate: base.playback_rate,
                mute: base.mute,
                volume: base.volume,
                hwdec: base.hwdec,
                power: base.power,
            }
        }
    }
}

/// Effective configuration after applying per-output overrides
#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub source: VideoSource,
    pub layout: LayoutMode,
    pub r#loop: bool,
    pub start_time: f64,
    pub playback_rate: f64,
    pub mute: bool,
    pub volume: f64,
    pub hwdec: bool,
    /// Power management config (for future Phase 6)
    #[allow(dead_code)]
    pub power: PowerConfig,
}

// Default value functions
fn default_layout() -> LayoutMode {
    LayoutMode::Fill
}

fn default_loop() -> bool {
    true
}

fn default_playback_rate() -> f64 {
    1.0
}

fn default_mute() -> bool {
    true
}

fn default_hwdec() -> bool {
    true
}

fn default_pause_hidden() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let yaml = r#"
source:
  type: File
  path: "/path/to/video.mp4"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.r#loop);
        assert_eq!(config.layout, LayoutMode::Fill);
    }

    #[test]
    fn test_per_output_override() {
        let yaml = r#"
source:
  type: File
  path: "/default.mp4"
layout: Fill
per_output:
  HDMI-A-1:
    layout: Contain
    start_time: 5.0
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let effective = config.for_output("HDMI-A-1");
        assert_eq!(effective.layout, LayoutMode::Contain);
        assert_eq!(effective.start_time, 5.0);
    }
}
