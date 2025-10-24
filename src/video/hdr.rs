use serde::{Deserialize, Serialize};

/// Color space of video content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ColorSpace {
    /// Standard Dynamic Range (BT.709)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransferFunction {
    /// Standard sRGB/BT.709
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HdrMode {
    /// Automatically detect and handle HDR (default)
    Auto,
    /// Force HDR processing
    Force,
    /// Disable HDR processing
    Disable,
}

impl Default for HdrMode {
    fn default() -> Self {
        HdrMode::Auto
    }
}

/// Tone mapping algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToneMappingAlgorithm {
    /// Hable (Uncharted 2) - Good for most content
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

impl Default for ToneMappingAlgorithm {
    fn default() -> Self {
        ToneMappingAlgorithm::Hable
    }
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
