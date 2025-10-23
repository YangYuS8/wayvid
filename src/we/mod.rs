// WE (Wallpaper Engine) module for importing video wallpapers

pub mod converter;
pub mod parser;
pub mod types;

pub use parser::{detect_we_project, parse_we_project};
