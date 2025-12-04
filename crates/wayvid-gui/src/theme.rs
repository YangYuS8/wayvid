//! Theme definitions for wayvid-gui
//!
//! Custom theme support with dark and light modes.

use iced::Theme;

/// Application theme
#[derive(Debug, Clone, Copy, Default)]
pub enum WayvidTheme {
    #[default]
    Dark,
    Light,
}

impl From<WayvidTheme> for Theme {
    fn from(theme: WayvidTheme) -> Self {
        match theme {
            WayvidTheme::Dark => Theme::Dark,
            WayvidTheme::Light => Theme::Light,
        }
    }
}
