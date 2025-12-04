//! wayvid-core: Core types and configuration for wayvid
//!
//! This crate provides the foundational types used across all wayvid components:
//! - `VideoSource`: Defines various video/wallpaper source types
//! - `WallpaperItem`: Wallpaper metadata for the library
//! - `AppSettings`: Application settings (GUI managed)
//! - `Config`: Legacy CLI configuration format
//! - Layout, HDR, and rendering types

pub mod config;
pub mod hdr;
pub mod layout;
pub mod library;
pub mod power;
pub mod settings;
pub mod types;

// Re-exports for convenience
pub use config::{Config, EffectiveConfig, OutputConfig, PowerConfig};
pub use hdr::{
    ColorSpace, HdrMetadata, HdrMode, ToneMappingAlgorithm, ToneMappingConfig, TransferFunction,
};
pub use layout::{calculate_layout, LayoutTransform};
pub use library::{SourceType, WallpaperItem, WallpaperMetadata, WallpaperType};
pub use settings::AppSettings;
pub use types::{HwdecMode, LayoutMode, OutputHdrCapabilities, OutputInfo, RenderBackend, VideoSource};
