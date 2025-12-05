//! View definitions
//!
//! Each view represents a major section of the application.

pub mod about;
pub mod folders;
pub mod library;
pub mod monitors;
pub mod settings;

use rust_i18n::t;

/// Application views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// Main wallpaper library grid
    #[default]
    Library,
    /// Folder management
    Folders,
    /// Monitor management (accessed via sidebar selector)
    #[allow(dead_code)]
    Monitors,
    /// Application settings
    Settings,
    /// About information
    About,
}

impl View {
    /// Get the title for this view (localized)
    pub fn title(&self) -> String {
        match self {
            Self::Library => t!("nav.library").to_string(),
            Self::Folders => t!("nav.folders").to_string(),
            Self::Monitors => t!("nav.monitors").to_string(),
            Self::Settings => t!("nav.settings").to_string(),
            Self::About => t!("nav.about").to_string(),
        }
    }
}
