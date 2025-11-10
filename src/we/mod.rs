// WE (Wallpaper Engine) module for importing video wallpapers

pub mod converter;
pub mod parser;
pub mod steam;
pub mod types;
pub mod workshop;

pub use parser::{detect_we_project, parse_we_project};
pub use steam::SteamLibrary;
pub use workshop::{WorkshopScanner, WALLPAPER_ENGINE_APP_ID};
