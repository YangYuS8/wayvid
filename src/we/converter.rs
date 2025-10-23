use crate::config::Config;
use crate::core::types::{LayoutMode, VideoSource};
use crate::we::types::{WeProject, WeProperties};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

/// Generate wayvid config from Wallpaper Engine project
pub fn generate_wayvid_config(project: &WeProject, video_path: PathBuf) -> Result<Config> {
    info!("🔄 Converting Wallpaper Engine project to wayvid config");

    // Extract properties
    let props = project.extract_properties();

    info!("📊 Extracted properties:");
    info!("  - Rate: {}", props.rate);
    info!("  - Volume: {}", props.volume);
    info!(
        "  - Playback mode: {} ({})",
        props.playback_mode,
        if props.playback_mode == 0 {
            "loop"
        } else {
            "pause"
        }
    );
    info!(
        "  - Alignment: {} ({})",
        props.alignment,
        alignment_name(props.alignment)
    );
    info!("  - Audio processing: {}", props.audio_processing);

    // Convert alignment to layout mode
    let layout = convert_alignment(props.alignment);

    // Convert playback mode to loop
    let loop_playback = props.playback_mode == 0;

    // Determine mute status
    let mute = props.volume == 0.0;

    // Warn about unsupported features
    if props.audio_processing {
        warn!("⚠️  Audio processing is enabled in WE project but not supported by wayvid");
    }

    if props.playback_mode != 0 && props.playback_mode != 1 {
        warn!(
            "⚠️  Unknown playback mode: {}. Defaulting to loop.",
            props.playback_mode
        );
    }

    // Create config
    let config = Config {
        source: VideoSource::File {
            path: video_path.to_string_lossy().to_string(),
        },
        layout,
        r#loop: loop_playback,
        start_time: 0.0,
        playback_rate: props.rate,
        mute,
        volume: props.volume,
        hwdec: true,
        per_output: HashMap::new(),
        power: Default::default(),
    };

    info!("✅ Config generated successfully");
    info!("  - Layout: {:?}", layout);
    info!("  - Loop: {}", loop_playback);
    info!("  - Mute: {}", mute);

    Ok(config)
}

/// Convert WE alignment value to wayvid LayoutMode
fn convert_alignment(alignment: i64) -> LayoutMode {
    match alignment {
        0 => LayoutMode::Centre,  // Center
        1 => LayoutMode::Contain, // Fit
        2 => LayoutMode::Cover,   // Fill
        3 => LayoutMode::Fill,    // Stretch
        _ => {
            warn!(
                "⚠️  Unknown alignment value: {}. Defaulting to Contain.",
                alignment
            );
            LayoutMode::Contain
        }
    }
}

/// Get human-readable alignment name
fn alignment_name(alignment: i64) -> &'static str {
    match alignment {
        0 => "center",
        1 => "fit",
        2 => "fill",
        3 => "stretch",
        _ => "unknown",
    }
}

/// Generate config with metadata comments
pub fn generate_config_with_metadata(project: &WeProject, video_path: PathBuf) -> Result<String> {
    let config = generate_wayvid_config(project, video_path)?;

    // Generate YAML with comments
    let mut yaml = String::new();

    // Add metadata header
    yaml.push_str("# wayvid configuration\n");
    yaml.push_str("# Imported from Wallpaper Engine\n");
    yaml.push_str(&format!("# Title: {}\n", project.title));

    if let Some(ref workshop_id) = project.workshopid {
        yaml.push_str(&format!("# Workshop ID: {}\n", workshop_id));
    }

    if !project.description.is_empty() {
        yaml.push_str(&format!("# Description: {}\n", project.description));
    }

    yaml.push('\n');

    // Serialize config
    let config_yaml = serde_yaml::to_string(&config)?;
    yaml.push_str(&config_yaml);

    Ok(yaml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::we::types::WeGeneral;
    use std::path::Path;

    #[test]
    fn test_convert_alignment() {
        assert_eq!(convert_alignment(0), LayoutMode::Centre);
        assert_eq!(convert_alignment(1), LayoutMode::Contain);
        assert_eq!(convert_alignment(2), LayoutMode::Cover);
        assert_eq!(convert_alignment(3), LayoutMode::Fill);
        assert_eq!(convert_alignment(99), LayoutMode::Contain); // Unknown defaults to Contain
    }

    #[test]
    fn test_generate_wayvid_config_minimal() {
        let project = WeProject {
            project_type: "video".to_string(),
            file: "video.mp4".to_string(),
            title: "Test".to_string(),
            description: "Test description".to_string(),
            preview: None,
            workshopid: None,
            general: WeGeneral::default(),
            tags: vec![],
        };

        let video_path = PathBuf::from("/path/to/video.mp4");
        let config = generate_wayvid_config(&project, video_path).unwrap();

        // Check defaults
        assert_eq!(config.playback_rate, 1.0);
        assert_eq!(config.volume, 50.0);
        assert!(config.loop_playback);
        assert_eq!(config.layout, LayoutMode::Contain);
    }

    #[test]
    fn test_generate_config_with_metadata() {
        let project = WeProject {
            project_type: "video".to_string(),
            file: "ocean.mp4".to_string(),
            title: "Ocean Waves".to_string(),
            description: "Beautiful ocean scene".to_string(),
            preview: None,
            workshopid: Some("123456789".to_string()),
            general: WeGeneral::default(),
            tags: vec!["Nature".to_string(), "Ocean".to_string()],
        };

        let video_path = PathBuf::from("/path/to/ocean.mp4");
        let yaml = generate_config_with_metadata(&project, video_path).unwrap();

        // Check metadata comments
        assert!(yaml.contains("# Title: Ocean Waves"));
        assert!(yaml.contains("# Workshop ID: 123456789"));
        assert!(yaml.contains("# Description: Beautiful ocean scene"));

        // Check config content
        assert!(yaml.contains("source:"));
        assert!(yaml.contains("layout:"));
    }
}
