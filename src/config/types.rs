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
    /// Priority for pattern matching (lower = higher priority)
    /// Default: 50 (mid priority)
    /// Exact matches always have priority 0
    #[serde(default = "default_priority")]
    pub priority: u32,

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
    /// Priority rules:
    /// 1. Exact matches (always priority 0)
    /// 2. Pattern matches sorted by priority (lower number = higher priority)
    /// 3. For same priority, more specific patterns win (fewer wildcards, then longer)
    pub fn for_output(&self, output_name: &str) -> EffectiveConfig {
        use crate::config::pattern::matches_pattern;

        let base = self.clone();

        // Collect all matching patterns with their priorities
        let mut matches: Vec<(&String, &OutputConfig, u32)> = self
            .per_output
            .iter()
            .filter(|(pattern, _)| matches_pattern(output_name, pattern))
            .map(|(pattern, config)| {
                // Calculate combined score for sorting
                let is_exact = pattern.as_str() == output_name;
                let wildcards = pattern.chars().filter(|&c| c == '*' || c == '?').count();

                let pattern_score = if is_exact {
                    0 // Exact match always wins
                } else {
                    // For patterns: priority (0-99) × 10000 + wildcards × 1000 - length
                    config.priority * 10000 + (wildcards as u32) * 1000 - (pattern.len() as u32)
                };

                (pattern, config, pattern_score)
            })
            .collect();

        if matches.is_empty() {
            // No match, use base config
            return EffectiveConfig {
                source: base.source,
                layout: base.layout,
                r#loop: base.r#loop,
                start_time: base.start_time,
                playback_rate: base.playback_rate,
                mute: base.mute,
                volume: base.volume,
                hwdec: base.hwdec,
                power: base.power,
            };
        }

        // Sort by score (lower = better)
        matches.sort_by_key(|(_, _, score)| *score);

        // Use best match
        let (_, override_cfg, _) = matches[0];
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

fn default_priority() -> u32 {
    50 // Mid priority (lower = higher priority)
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

    #[test]
    fn test_priority_sorting() {
        let yaml = r#"
source:
  type: File
  path: "/default.mp4"
layout: Fill
per_output:
  "*":
    priority: 99
    layout: Stretch
  "HDMI-*":
    priority: 10
    layout: Contain
  "HDMI-A-*":
    priority: 5
    layout: Cover
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        // HDMI-A-1 matches all three, but HDMI-A-* has highest priority (5)
        let effective = config.for_output("HDMI-A-1");
        assert_eq!(effective.layout, LayoutMode::Cover);

        // HDMI-B-1 matches HDMI-* (priority 10) and * (priority 99)
        let effective = config.for_output("HDMI-B-1");
        assert_eq!(effective.layout, LayoutMode::Contain);

        // DP-1 only matches * (priority 99)
        let effective = config.for_output("DP-1");
        assert_eq!(effective.layout, LayoutMode::Stretch);
    }

    #[test]
    fn test_exact_match_priority() {
        let yaml = r#"
source:
  type: File
  path: "/default.mp4"
layout: Fill
per_output:
  "HDMI-A-1":
    priority: 50
    layout: Centre
  "HDMI-*":
    priority: 1
    layout: Contain
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        // Exact match always wins, even with higher priority number
        let effective = config.for_output("HDMI-A-1");
        assert_eq!(effective.layout, LayoutMode::Centre);

        // Pattern match
        let effective = config.for_output("HDMI-A-2");
        assert_eq!(effective.layout, LayoutMode::Contain);
    }

    #[test]
    fn test_fallback_pattern() {
        let yaml = r#"
source:
  type: File
  path: "/default.mp4"
layout: Fill
per_output:
  "HDMI-*":
    layout: Contain
  "*":
    priority: 99
    layout: Stretch
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        // HDMI matches specific pattern
        let effective = config.for_output("HDMI-A-1");
        assert_eq!(effective.layout, LayoutMode::Contain);

        // Others match fallback *
        let effective = config.for_output("DP-1");
        assert_eq!(effective.layout, LayoutMode::Stretch);

        let effective = config.for_output("eDP-1");
        assert_eq!(effective.layout, LayoutMode::Stretch);
    }
}
