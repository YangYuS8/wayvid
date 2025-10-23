// WE (Wallpaper Engine) module for importing video wallpapers

pub mod converter;
pub mod parser;
pub mod types;

pub use converter::generate_wayvid_config;
pub use parser::{detect_we_project, parse_we_project};
pub use types::{WeProject, WeProperties};
