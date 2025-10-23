use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Wallpaper Engine project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeProject {
    /// Project type (must be "video" for import)
    #[serde(rename = "type")]
    pub project_type: String,

    /// Video file path (relative to project directory)
    pub file: String,

    /// Project title
    #[serde(default)]
    pub title: String,

    /// Project description
    #[serde(default)]
    pub description: String,

    /// Preview image path
    #[serde(default)]
    pub preview: Option<String>,

    /// Steam Workshop ID
    #[serde(default)]
    pub workshopid: Option<String>,

    /// General settings and properties
    #[serde(default)]
    pub general: WeGeneral,

    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// General settings container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeGeneral {
    /// Properties dictionary
    #[serde(default)]
    pub properties: HashMap<String, WeProperty>,
}

/// Generic property structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WeProperty {
    /// Combo/dropdown property (integer enum)
    /// MUST be before Slider to avoid f64 matching integers
    Combo {
        #[serde(default)]
        order: i32,
        #[serde(default)]
        text: String,
        #[serde(default)]
        r#type: String,
        value: i64,
        #[serde(default)]
        options: Vec<WeComboOption>,
    },

    /// Slider property (float value)
    Slider {
        #[serde(default)]
        order: i32,
        #[serde(default)]
        text: String,
        #[serde(default)]
        r#type: String,
        value: f64,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        fraction: Option<bool>,
    },

    /// Boolean property
    Bool {
        #[serde(default)]
        order: i32,
        #[serde(default)]
        text: String,
        #[serde(default)]
        r#type: String,
        value: bool,
    },

    /// Color property (RGB string)
    Color {
        #[serde(default)]
        order: i32,
        #[serde(default)]
        text: String,
        #[serde(default)]
        r#type: String,
        value: String,
    },
}

/// Combo option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeComboOption {
    pub label: String,
    pub value: i64,
}

/// Extracted video properties (simplified)
#[derive(Debug, Clone)]
pub struct WeProperties {
    /// Playback rate (0.1 - 5.0)
    pub rate: f64,

    /// Volume (0.0 - 100.0)
    pub volume: f64,

    /// Playback mode (0 = loop, 1 = pause)
    pub playback_mode: i64,

    /// Alignment/layout (0 = center, 1 = fit, 2 = fill, 3 = stretch)
    pub alignment: i64,

    /// Audio processing enabled
    pub audio_processing: bool,

    /// Scheme color (RGB string, e.g., "0.2 0.4 0.6")
    pub scheme_color: Option<String>,
}

impl Default for WeProperties {
    fn default() -> Self {
        Self {
            rate: 1.0,
            volume: 50.0,
            playback_mode: 0, // Loop by default
            alignment: 1,     // Fit by default
            audio_processing: false,
            scheme_color: None,
        }
    }
}

impl WeProject {
    /// Extract simplified properties for conversion
    pub fn extract_properties(&self) -> WeProperties {
        let props = &self.general.properties;

        let rate = props
            .get("rate")
            .and_then(|p| match p {
                WeProperty::Slider { value, .. } => Some(*value),
                _ => None,
            })
            .unwrap_or(1.0);

        let volume = props
            .get("volume")
            .and_then(|p| match p {
                WeProperty::Slider { value, .. } => Some(*value),
                _ => None,
            })
            .unwrap_or(50.0);

        let playback_mode = props
            .get("playbackmode")
            .and_then(|p| match p {
                WeProperty::Combo { value, .. } => Some(*value),
                _ => None,
            })
            .unwrap_or(0);

        let alignment = props
            .get("alignment")
            .and_then(|p| match p {
                WeProperty::Combo { value, .. } => Some(*value),
                _ => None,
            })
            .unwrap_or(1);

        let audio_processing = props
            .get("audioprocessing")
            .and_then(|p| match p {
                WeProperty::Bool { value, .. } => Some(*value),
                _ => None,
            })
            .unwrap_or(false);

        let scheme_color = props.get("schemecolor").and_then(|p| match p {
            WeProperty::Color { value, .. } => Some(value.clone()),
            _ => None,
        });

        WeProperties {
            rate,
            volume,
            playback_mode,
            alignment,
            audio_processing,
            scheme_color,
        }
    }
}
