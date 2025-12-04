//! Application state management
//!
//! Contains the global state shared across views.

use std::collections::HashMap;
use std::path::PathBuf;

use iced::Task;
use serde::{Deserialize, Serialize};
use wayvid_core::{WallpaperItem, WallpaperType};

use crate::messages::Message;
use crate::views::View;

/// Main application state
#[derive(Debug)]
pub struct AppState {
    /// Current view
    pub current_view: View,
    
    /// All wallpapers in the library
    pub wallpapers: Vec<WallpaperItem>,
    
    /// Currently selected wallpaper ID
    pub selected_wallpaper: Option<String>,
    
    /// Registered folders
    pub folders: Vec<FolderEntry>,
    
    /// Search query
    pub search_query: String,
    
    /// Current filter
    pub current_filter: WallpaperFilter,
    
    /// Loading indicator
    pub loading: bool,
    
    /// Current error message
    pub error: Option<String>,
    
    /// Status message
    pub status_message: Option<String>,
    
    /// Daemon connection status
    pub daemon_connected: bool,
    
    /// Cached thumbnails (wallpaper_id -> image data)
    pub thumbnails: HashMap<String, Vec<u8>>,
    
    /// Application settings
    pub settings: Settings,
    
    /// Connected monitors
    pub monitors: Vec<MonitorInfo>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            current_view: View::Library,
            wallpapers: Vec::new(),
            selected_wallpaper: None,
            folders: Vec::new(),
            search_query: String::new(),
            current_filter: WallpaperFilter::All,
            loading: false,
            error: None,
            status_message: None,
            daemon_connected: false,
            thumbnails: HashMap::new(),
            settings: Settings::default(),
            monitors: Vec::new(),
        };

        // Load library on startup
        let task = Task::perform(async {}, |_| Message::LoadLibrary);

        (state, task)
    }

    /// Get filtered wallpapers based on current search and filter
    pub fn filtered_wallpapers(&self) -> Vec<&WallpaperItem> {
        self.wallpapers
            .iter()
            .filter(|wp| {
                // Apply type filter
                let type_match = match self.current_filter {
                    WallpaperFilter::All => true,
                    WallpaperFilter::Videos => matches!(wp.wallpaper_type, WallpaperType::Video),
                    WallpaperFilter::Images => matches!(wp.wallpaper_type, WallpaperType::Image),
                    WallpaperFilter::Gifs => matches!(wp.wallpaper_type, WallpaperType::Gif),
                    WallpaperFilter::Scenes => matches!(wp.wallpaper_type, WallpaperType::Scene),
                    WallpaperFilter::Favorites => false, // TODO: implement favorites
                };

                // Apply search query
                let search_match = if self.search_query.is_empty() {
                    true
                } else {
                    let query = self.search_query.to_lowercase();
                    wp.name.to_lowercase().contains(&query)
                        || wp.metadata.title.as_ref()
                            .map(|t| t.to_lowercase().contains(&query))
                            .unwrap_or(false)
                        || wp.metadata.tags.iter().any(|t| t.to_lowercase().contains(&query))
                };

                type_match && search_match
            })
            .collect()
    }
}

/// Folder entry in the library
#[derive(Debug, Clone)]
pub struct FolderEntry {
    /// Folder path
    pub path: PathBuf,
    /// Whether the folder is enabled for scanning
    pub enabled: bool,
    /// Number of wallpapers found
    pub wallpaper_count: usize,
    /// Last scan time
    pub last_scan: Option<String>,
}

/// Wallpaper filter options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WallpaperFilter {
    #[default]
    All,
    Videos,
    Images,
    Gifs,
    Scenes,
    Favorites,
}

impl WallpaperFilter {
    /// Get display name for the filter
    pub fn name(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Videos => "Videos",
            Self::Images => "Images",
            Self::Gifs => "GIFs",
            Self::Scenes => "Scenes",
            Self::Favorites => "Favorites",
        }
    }

    /// Get all filter variants
    pub fn all() -> &'static [WallpaperFilter] {
        &[
            Self::All,
            Self::Videos,
            Self::Images,
            Self::Gifs,
            Self::Scenes,
            Self::Favorites,
        ]
    }
}

/// Monitor information
#[derive(Debug, Clone, Default)]
pub struct MonitorInfo {
    /// Monitor name/identifier (e.g. "eDP-1")
    pub name: String,
    /// Monitor width in pixels
    pub width: u32,
    /// Monitor height in pixels
    pub height: u32,
    /// X position
    pub x: i32,
    /// Y position
    pub y: i32,
    /// Scale factor
    pub scale: f64,
    /// Whether this is the primary monitor
    pub primary: bool,
    /// Currently applied wallpaper path
    pub current_wallpaper: Option<std::path::PathBuf>,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Start with system
    pub autostart: bool,
    /// Minimize to tray on close
    pub minimize_to_tray: bool,
    /// Default playback volume
    pub volume: f32,
    /// Pause on battery
    pub pause_on_battery: bool,
    /// Pause when fullscreen app is running
    pub pause_on_fullscreen: bool,
    /// FPS limit
    pub fps_limit: Option<u32>,
    /// Preferred monitor
    pub preferred_monitor: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: false,
            minimize_to_tray: true,
            volume: 0.0, // Muted by default
            pause_on_battery: true,
            pause_on_fullscreen: true,
            fps_limit: None,
            preferred_monitor: None,
        }
    }
}

/// Type alias for state used in views
pub type WayvidState = AppState;
