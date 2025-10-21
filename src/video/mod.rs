pub mod egl;

#[cfg(feature = "video-mpv")]
pub mod mpv;

// Re-export for convenience
pub use egl::{EglContext, EglWindow};
