// WE (Wallpaper Engine) module for importing video wallpapers

// Allow unused re-exports as they are part of the public API
#![allow(unused_imports)]

pub mod converter;
pub mod downloader;
pub mod parser;
pub mod scene;
pub mod steam;
pub mod types;
pub mod workshop;

pub use downloader::WorkshopDownloader;
pub use parser::{
    detect_we_project, detect_we_project_type, parse_we_project, parse_we_project_any,
    ParsedWeProject, WeProjectType,
};
pub use scene::{SceneParser, SceneProject, SceneRenderer};
pub use steam::SteamLibrary;
pub use workshop::{WorkshopScanner, WALLPAPER_ENGINE_APP_ID};
