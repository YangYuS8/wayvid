//! Wayland backend for layer-shell surface management
//!
//! This module handles Wayland protocol interactions for creating
//! background wallpaper surfaces using wlr-layer-shell.

// TODO: Migrate full implementation from src/backend/wayland/
// For now, this is a placeholder module structure

pub mod layer_shell;
pub mod output;

pub use layer_shell::LayerSurface;
pub use output::OutputManager;
