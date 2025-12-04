//! wayvid-gui: Graphical user interface for wayvid
//!
//! A modern wallpaper manager GUI built with iced.
//!
//! Features:
//! - Wallpaper library browsing with thumbnails
//! - Folder management
//! - Settings configuration
//! - Real-time wallpaper preview
//! - Steam Workshop integration

// Initialize rust-i18n at crate root
rust_i18n::i18n!("locales");

mod app;
mod async_loader;
mod i18n;
mod ipc;
mod messages;
mod service;
mod settings;
mod state;
mod theme;
mod views;
mod widgets;

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("wayvid_gui=info".parse()?))
        .init();

    tracing::info!("Starting wayvid-gui v{}", env!("CARGO_PKG_VERSION"));

    // Initialize internationalization
    i18n::init();

    // Run the GUI application
    app::run()
}
