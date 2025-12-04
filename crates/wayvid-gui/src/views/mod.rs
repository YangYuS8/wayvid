//! View definitions
//!
//! Each view represents a major section of the application.

pub mod library;
pub mod folders;
pub mod settings;
pub mod about;
pub mod monitors;

/// Application views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// Main wallpaper library grid
    #[default]
    Library,
    /// Folder management
    Folders,
    /// Monitor management
    Monitors,
    /// Application settings
    Settings,
    /// About information
    About,
}

impl View {
    /// Get the title for this view
    pub fn title(&self) -> &'static str {
        match self {
            Self::Library => "Library",
            Self::Folders => "Folders",
            Self::Monitors => "Monitors",
            Self::Settings => "Settings",
            Self::About => "About",
        }
    }
}
