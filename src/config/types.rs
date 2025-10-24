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

    /// Maximum memory usage in MB (0 = unlimited)
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: usize,

    /// Maximum number of texture buffers in pool
    #[serde(default = "default_max_buffers")]
    pub max_buffers: usize,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            pause_when_hidden: true,
            pause_on_battery: false,
            max_fps: 0,
            max_memory_mb: default_max_memory_mb(),
            max_buffers: default_max_buffers(),
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
    ///
    /// Supports pattern matching (e.g., "HDMI-*", "DP-?") for flexible configuration.
    /// Exact matches take precedence over pattern matches.
    pub fn for_output(&self, output_name: &str) -> EffectiveConfig {
        use crate::config::pattern::find_best_match;

        let base = self.clone();

        // Try exact match first, then pattern matching
        let matching_key = if self.per_output.contains_key(output_name) {
            Some(output_name.to_string())
        } else {
            // Collect all keys and find best pattern match
            let patterns: Vec<&str> = self.per_output.keys().map(|s| s.as_str()).collect();
            find_best_match(output_name, &patterns).map(|s| s.to_string())
        };

        if let Some(key) = matching_key {
            if let Some(override_cfg) = self.per_output.get(&key) {
                return EffectiveConfig {
                    source: override_cfg.source.clone().unwrap_or(base.source),
                    layout: override_cfg.layout.unwrap_or(base.layout),
                    r#loop: base.r#loop,
                    start_time: override_cfg.start_time.unwrap_or(base.start_time),
                    playback_rate: override_cfg.playback_rate.unwrap_or(base.playback_rate),
                    mute: override_cfg.mute.unwrap_or(base.mute),
                    volume: override_cfg.volume.unwrap_or(base.volume),
                    hwdec: base.hwdec,
                    power: base.power.clone(),
                };
            }
        }

        // No match, use base config
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

fn default_max_memory_mb() -> usize {
    100 // 100MB default limit
}

fn default_max_buffers() -> usize {
    8 // Default 8 texture buffers
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

    #[test]
    fn test_pattern_matching() {
        let yaml = r#"
source:
  type: File
  path: "/default.mp4"
layout: Fill
per_output:
  "HDMI-*":
    layout: Contain
  "DP-?":
    layout: Stretch
  "eDP-1":
    layout: Cover
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        // Exact match takes precedence
        let effective = config.for_output("eDP-1");
        assert_eq!(effective.layout, LayoutMode::Cover);

        // Pattern match: HDMI-*
        let effective = config.for_output("HDMI-A-1");
        assert_eq!(effective.layout, LayoutMode::Contain);

        let effective = config.for_output("HDMI-B-2");
        assert_eq!(effective.layout, LayoutMode::Contain);

        // Pattern match: DP-?
        let effective = config.for_output("DP-1");
        assert_eq!(effective.layout, LayoutMode::Stretch);

        let effective = config.for_output("DP-2");
        assert_eq!(effective.layout, LayoutMode::Stretch);

        // No match: fallback to base
        let effective = config.for_output("DVI-I-1");
        assert_eq!(effective.layout, LayoutMode::Fill);
    }
}
