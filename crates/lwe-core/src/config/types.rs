//! Configuration types for CLI (legacy config.yaml format)
//!
//! This module provides backwards compatibility with the CLI-based configuration.
//! For new GUI-first approach, use `AppSettings` instead.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::hdr::{HdrMode, ToneMappingConfig};
use crate::types::{LayoutMode, RenderBackend, VideoSource};

use super::pattern::matches_pattern;

/// Global configuration (legacy CLI format)
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

    /// HDR mode (auto/force/disable)
    #[serde(default)]
    pub hdr_mode: HdrMode,

    /// Tone mapping configuration
    #[serde(default)]
    pub tone_mapping: ToneMappingConfig,

    /// Render backend selection (auto, opengl, vulkan)
    #[serde(default)]
    pub render_backend: RenderBackend,

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
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).with_context(|| {
            format!(
                "Cannot read configuration file: {:?}\n\
                 Please check file exists and is readable.",
                path_ref
            )
        })?;

        let mut config: Self = serde_yaml::from_str(&content).with_context(|| {
            format!(
                "Invalid YAML syntax in configuration: {:?}\n\
                 Please check YAML syntax.",
                path_ref
            )
        })?;

        // Validate and fix configuration
        config.validate();

        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = serde_yaml::to_string(self).context("Failed to serialize config")?;
        fs::write(path_ref, content)
            .with_context(|| format!("Failed to write config file: {:?}", path_ref))?;

        Ok(())
    }

    /// Update source for a specific output and save
    pub fn set_output_source(&mut self, output_name: &str, source: VideoSource) {
        if let Some(output_config) = self.per_output.get_mut(output_name) {
            output_config.source = Some(source);
        } else {
            self.per_output.insert(
                output_name.to_string(),
                OutputConfig {
                    priority: default_priority(),
                    source: Some(source),
                    layout: None,
                    start_time: None,
                    playback_rate: None,
                    mute: None,
                    volume: None,
                },
            );
        }
    }

    /// Validate and fix configuration values
    fn validate(&mut self) {
        // Validate tone mapping config
        self.tone_mapping.validate();

        // Validate playback rate
        if self.playback_rate <= 0.0 || self.playback_rate > 100.0 {
            self.playback_rate = self.playback_rate.clamp(0.1, 10.0);
        }

        // Validate volume
        if self.volume < 0.0 || self.volume > 1.0 {
            self.volume = self.volume.clamp(0.0, 1.0);
        }

        // Validate start_time
        if self.start_time < 0.0 {
            self.start_time = 0.0;
        }
    }

    /// Get effective configuration for a specific output
    ///
    /// Supports pattern matching (e.g., "HDMI-*", "DP-?") for flexible configuration.
    pub fn for_output(&self, output_name: &str) -> EffectiveConfig {
        let base = self.clone();

        // Collect all matching patterns with their priorities
        let mut matches: Vec<(&String, &OutputConfig, u32)> = self
            .per_output
            .iter()
            .filter(|(pattern, _)| matches_pattern(output_name, pattern))
            .map(|(pattern, config)| {
                let is_exact = pattern.as_str() == output_name;
                let wildcards = pattern.chars().filter(|&c| c == '*' || c == '?').count();

                let pattern_score = if is_exact {
                    0
                } else {
                    config.priority * 10000 + (wildcards as u32) * 1000 - (pattern.len() as u32)
                };

                (pattern, config, pattern_score)
            })
            .collect();

        if matches.is_empty() {
            return EffectiveConfig {
                source: base.source,
                layout: base.layout,
                r#loop: base.r#loop,
                start_time: base.start_time,
                playback_rate: base.playback_rate,
                mute: base.mute,
                volume: base.volume,
                hwdec: base.hwdec,
                hdr_mode: base.hdr_mode,
                tone_mapping: base.tone_mapping.clone(),
                render_backend: base.render_backend,
                power: base.power,
            };
        }

        // Sort by score (lower = better)
        matches.sort_by_key(|(_, _, score)| *score);

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
            hdr_mode: base.hdr_mode,
            tone_mapping: base.tone_mapping.clone(),
            render_backend: base.render_backend,
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
    pub hdr_mode: HdrMode,
    pub tone_mapping: ToneMappingConfig,
    pub render_backend: RenderBackend,
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
    100
}

fn default_max_buffers() -> usize {
    8
}

fn default_priority() -> u32 {
    50
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
