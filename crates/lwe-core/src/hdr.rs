//! HDR types and utilities
//!
//! Color space, transfer function, and tone mapping configuration for HDR video support.

use serde::{Deserialize, Serialize};

/// Color space of video content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum ColorSpace {
    /// Standard Dynamic Range (BT.709)
    #[default]
    Sdr,
    /// High Dynamic Range (BT.2020)
    Hdr10,
    /// Hybrid Log-Gamma
    Hlg,
    /// Dolby Vision (for future support)
    DolbyVision,
    /// Unknown or unsupported
    Unknown,
}

impl ColorSpace {
    /// Check if this is an HDR color space
    pub fn is_hdr(&self) -> bool {
        matches!(
            self,
            ColorSpace::Hdr10 | ColorSpace::Hlg | ColorSpace::DolbyVision
        )
    }
}

/// Transfer function (gamma/EOTF)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum TransferFunction {
    /// Standard sRGB/BT.709
    #[default]
    Srgb,
    /// Perceptual Quantizer (HDR10)
    Pq,
    /// Hybrid Log-Gamma
    Hlg,
    /// Unknown or unsupported
    Unknown,
}

impl TransferFunction {
    /// Check if this is an HDR transfer function
    pub fn is_hdr(&self) -> bool {
        matches!(self, TransferFunction::Pq | TransferFunction::Hlg)
    }
}

/// HDR metadata from video stream
#[derive(Debug, Clone, Default)]
pub struct HdrMetadata {
    /// Color space (BT.709, BT.2020, etc.)
    pub color_space: ColorSpace,

    /// Transfer function (EOTF)
    pub transfer_function: TransferFunction,

    /// Color primaries string (e.g., "bt.709", "bt.2020")
    pub primaries: String,

    /// Signal peak luminance in nits (if available)
    pub peak_luminance: Option<f64>,

    /// Average luminance in nits (if available)
    pub avg_luminance: Option<f64>,

    /// Minimum luminance in nits (if available)
    pub min_luminance: Option<f64>,
}

impl HdrMetadata {
    /// Check if this video contains HDR content
    pub fn is_hdr(&self) -> bool {
        self.color_space.is_hdr() || self.transfer_function.is_hdr()
    }

    /// Get a human-readable description of the HDR format
    pub fn format_description(&self) -> String {
        if !self.is_hdr() {
            return "SDR".to_string();
        }

        match (&self.color_space, &self.transfer_function) {
            (ColorSpace::Hdr10, TransferFunction::Pq) => "HDR10",
            (ColorSpace::Hlg, TransferFunction::Hlg) => "HLG",
            (ColorSpace::DolbyVision, _) => "Dolby Vision",
            _ => "HDR (Unknown)",
        }
        .to_string()
    }
}

/// Parse MPV colorspace property to ColorSpace enum
pub fn parse_colorspace(value: &str) -> ColorSpace {
    match value.to_lowercase().as_str() {
        "bt.709" | "srgb" => ColorSpace::Sdr,
        "bt.2020-ncl" | "bt.2020-cl" | "bt2020" => ColorSpace::Hdr10,
        _ => ColorSpace::Unknown,
    }
}

/// Parse MPV gamma/transfer property to TransferFunction enum
pub fn parse_transfer_function(value: &str) -> TransferFunction {
    match value.to_lowercase().as_str() {
        "srgb" | "bt.1886" | "bt.709" => TransferFunction::Srgb,
        "pq" | "smpte2084" | "st2084" => TransferFunction::Pq,
        "hlg" | "arib-std-b67" => TransferFunction::Hlg,
        _ => TransferFunction::Unknown,
    }
}

/// HDR mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HdrMode {
    /// Automatically detect and handle HDR (default)
    #[default]
    Auto,
    /// Force HDR processing
    Force,
    /// Disable HDR processing
    Disable,
}

/// Tone mapping algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToneMappingAlgorithm {
    /// Hable (Uncharted 2) - Good for most content
    #[default]
    Hable,
    /// Mobius - Preserves details
    Mobius,
    /// Reinhard - Classic algorithm
    Reinhard,
    /// BT.2390 EETF - ITU standard
    Bt2390,
    /// Clip (no tone mapping)
    Clip,
}

impl ToneMappingAlgorithm {
    pub fn as_mpv_str(&self) -> &'static str {
        match self {
            ToneMappingAlgorithm::Hable => "hable",
            ToneMappingAlgorithm::Mobius => "mobius",
            ToneMappingAlgorithm::Reinhard => "reinhard",
            ToneMappingAlgorithm::Bt2390 => "bt.2390",
            ToneMappingAlgorithm::Clip => "clip",
        }
    }

    /// Get recommended parameter value for this algorithm
    pub fn recommended_param(&self) -> f64 {
        match self {
            ToneMappingAlgorithm::Hable => 1.0,
            ToneMappingAlgorithm::Mobius => 0.3,
            ToneMappingAlgorithm::Reinhard => 0.5,
            ToneMappingAlgorithm::Bt2390 => 1.0,
            ToneMappingAlgorithm::Clip => 1.0,
        }
    }

    /// Get description of this algorithm
    pub fn description(&self) -> &'static str {
        match self {
            ToneMappingAlgorithm::Hable => {
                "Hable (Uncharted 2) - Best overall quality, good contrast"
            }
            ToneMappingAlgorithm::Mobius => "Mobius - Preserves highlight details, softer look",
            ToneMappingAlgorithm::Reinhard => "Reinhard - Classic, simple, fast",
            ToneMappingAlgorithm::Bt2390 => "BT.2390 - ITU standard, broadcasting reference",
            ToneMappingAlgorithm::Clip => "Clip - No tone mapping, simple clipping",
        }
    }

    /// Check if this algorithm benefits from tone-mapping-param adjustment
    pub fn uses_param(&self) -> bool {
        !matches!(
            self,
            ToneMappingAlgorithm::Clip | ToneMappingAlgorithm::Bt2390
        )
    }
}

/// Tone mapping configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToneMappingConfig {
    /// Tone mapping algorithm
    #[serde(default)]
    pub algorithm: ToneMappingAlgorithm,

    /// Algorithm parameter (default: 1.0)
    #[serde(default = "default_tone_mapping_param")]
    pub param: f64,

    /// Enable dynamic peak detection
    #[serde(default = "default_compute_peak")]
    pub compute_peak: bool,

    /// Tone mapping mode: auto, rgb, hybrid, luma
    #[serde(default = "default_tone_mapping_mode")]
    pub mode: String,
}

impl Default for ToneMappingConfig {
    fn default() -> Self {
        Self {
            algorithm: ToneMappingAlgorithm::default(),
            param: default_tone_mapping_param(),
            compute_peak: default_compute_peak(),
            mode: default_tone_mapping_mode(),
        }
    }
}

fn default_tone_mapping_param() -> f64 {
    1.0
}

fn default_compute_peak() -> bool {
    true
}

fn default_tone_mapping_mode() -> String {
    "hybrid".to_string()
}

/// Content type for HDR optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// General purpose content
    General,
    /// High-contrast cinematic content
    Cinema,
    /// Animation with vibrant colors
    Animation,
    /// Nature/documentary with wide dynamic range
    Documentary,
    /// Low dynamic range content
    LowDynamicRange,
}

impl ContentType {
    /// Detect content type from HDR metadata
    pub fn detect_from_metadata(metadata: &HdrMetadata) -> Self {
        match metadata.peak_luminance {
            Some(peak) if peak > 2000.0 => ContentType::Cinema,
            Some(peak) if peak > 1000.0 => ContentType::Documentary,
            Some(peak) if peak < 400.0 => ContentType::LowDynamicRange,
            _ => ContentType::General,
        }
    }

    /// Get recommended tone mapping algorithm for this content type
    pub fn recommended_algorithm(&self) -> ToneMappingAlgorithm {
        match self {
            ContentType::General => ToneMappingAlgorithm::Hable,
            ContentType::Cinema => ToneMappingAlgorithm::Hable,
            ContentType::Animation => ToneMappingAlgorithm::Mobius,
            ContentType::Documentary => ToneMappingAlgorithm::Bt2390,
            ContentType::LowDynamicRange => ToneMappingAlgorithm::Reinhard,
        }
    }

    /// Get recommended tone mapping parameter for this content type
    pub fn recommended_param(&self, algorithm: ToneMappingAlgorithm) -> f64 {
        match (self, algorithm) {
            (ContentType::Cinema, ToneMappingAlgorithm::Hable) => 1.2,
            (ContentType::Animation, ToneMappingAlgorithm::Mobius) => 0.25,
            (ContentType::Documentary, ToneMappingAlgorithm::Bt2390) => 1.0,
            (ContentType::LowDynamicRange, ToneMappingAlgorithm::Reinhard) => 0.4,
            _ => algorithm.recommended_param(),
        }
    }
}

impl ToneMappingConfig {
    /// Apply content-aware optimizations based on HDR metadata
    pub fn optimize_for_content(&mut self, metadata: &HdrMetadata) {
        let content_type = ContentType::detect_from_metadata(metadata);

        // If using default param (1.0), apply content-aware optimization
        if (self.param - 1.0).abs() < 0.01 {
            self.param = content_type.recommended_param(self.algorithm);
        }

        // Adjust tone mapping mode based on content
        if self.mode == "hybrid" {
            self.mode = match content_type {
                ContentType::Cinema => "rgb".to_string(),
                ContentType::Animation => "luma".to_string(),
                ContentType::Documentary => "auto".to_string(),
                _ => "hybrid".to_string(),
            };
        }
    }

    /// Validate and clamp configuration values
    pub fn validate(&mut self) {
        // Clamp param to reasonable range
        self.param = self.param.clamp(0.0, 10.0);

        // Validate mode
        let valid_modes = ["auto", "rgb", "hybrid", "luma", "max"];
        if !valid_modes.contains(&self.mode.as_str()) {
            self.mode = "hybrid".to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdr_detection() {
        let mut metadata = HdrMetadata::default();
        assert!(!metadata.is_hdr());

        metadata.color_space = ColorSpace::Hdr10;
        metadata.transfer_function = TransferFunction::Pq;
        assert!(metadata.is_hdr());
        assert_eq!(metadata.format_description(), "HDR10");
    }

    #[test]
    fn test_tone_mapping_validation() {
        let mut config = ToneMappingConfig {
            param: 15.0,
            mode: "invalid".to_string(),
            ..Default::default()
        };
        config.validate();
        assert_eq!(config.param, 10.0);
        assert_eq!(config.mode, "hybrid");
    }
}
