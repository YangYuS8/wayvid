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

mod app;
mod async_loader;
mod ipc;
mod messages;
mod service;
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

    // Run the GUI application
    app::run()
}
