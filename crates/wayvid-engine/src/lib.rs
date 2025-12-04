//! wayvid-engine: Video rendering engine for wayvid
//!
//! This crate provides the core rendering functionality:
//! - Wayland layer-shell surface management
//! - MPV video playback integration  
//! - EGL/OpenGL rendering
//! - Vulkan rendering (optional)
//!
//! # Architecture
//!
//! ```text
//! WallpaperEngine
//!     ├── WaylandBackend (surface management)
//!     │   └── LayerSurface (per-output)
//!     └── VideoPlayer (per-surface)
//!         ├── MpvPlayer (video decoding)
//!         └── EglContext (OpenGL rendering)
//! ```

pub mod egl;
pub mod frame_timing;
pub mod mpv;
pub mod wayland;

// Re-exports
pub use egl::{EglContext, EglWindow};
pub use frame_timing::FrameTiming;
pub use mpv::{MpvPlayer, VideoConfig};
pub use wayland::{LayerSurface, OutputManager};

// Re-exports from wayvid-core
pub use wayvid_core::{
    calculate_layout, HdrMetadata, HdrMode, HwdecMode, LayoutMode,
    LayoutTransform, OutputInfo, RenderBackend, ToneMappingConfig,
};
