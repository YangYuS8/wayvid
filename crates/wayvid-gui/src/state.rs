//! Application state management
//!
//! Contains the global state shared across views.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use iced::Task;
use wayvid_core::{WallpaperItem, WallpaperType};

use crate::async_loader::{AsyncLoader, ThumbnailRequest};
use crate::i18n::{self, Language};
use crate::ipc::{ConnectionState, DaemonStatus};
use crate::messages::Message;
use crate::settings::{AppSettings, AutostartManager};
use crate::views::View;

/// Thumbnail loading state for a wallpaper
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThumbnailState {
    /// Thumbnail not yet requested
    NotLoaded,
    /// Thumbnail loading in progress
    Loading,
    /// Thumbnail loaded successfully (data in thumbnails HashMap)
    Loaded,
    /// Thumbnail loading failed
    Failed(String),
}

/// Main application state
#[derive(Debug)]
pub struct AppState {
    /// Current view
    pub current_view: View,

    /// All wallpapers in the library (Workshop + Local)
    pub wallpapers: Vec<WallpaperItem>,

    /// Workshop wallpapers (primary source)
    pub workshop_wallpapers: Vec<WallpaperItem>,

    /// Local folder wallpapers (secondary source)
    pub local_wallpapers: Vec<WallpaperItem>,

    /// Currently selected wallpaper ID
    pub selected_wallpaper: Option<String>,

    /// Registered folders
    pub folders: Vec<FolderEntry>,

    /// Search query
    pub search_query: String,

    /// Current filter
    pub current_filter: WallpaperFilter,

    /// Source filter (Workshop, Local, All)
    pub source_filter: SourceFilter,

    /// Loading indicator
    pub loading: bool,

    /// Workshop scan in progress
    pub workshop_scanning: bool,

    /// Current error message
    pub error: Option<String>,

    /// Status message
    pub status_message: Option<String>,

    /// Daemon connection status
    pub daemon_connected: bool,

    /// Cached thumbnails (wallpaper_id -> image data)
    pub thumbnails: HashMap<String, Vec<u8>>,

    /// Thumbnail loading states (wallpaper_id -> state)
    pub thumbnail_states: HashMap<String, ThumbnailState>,

    /// Pending thumbnail requests (wallpaper IDs being loaded)
    pub pending_thumbnails: HashSet<String>,

    /// Async loader for thumbnail generation
    pub async_loader: AsyncLoader,

    /// Application settings (persisted)
    pub app_settings: AppSettings,

    /// Connected monitors
    pub monitors: Vec<MonitorInfo>,

    /// Whether Steam/Workshop was found
    pub workshop_available: bool,

    /// IPC connection state
    pub ipc_state: ConnectionState,

    /// Last daemon status received
    pub last_daemon_status: Option<DaemonStatus>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> (Self, Task<Message>) {
        // Check if Workshop is available
        let workshop_available = wayvid_library::SteamLibrary::try_discover().is_some();

        // Initialize async loader (creates cache dir)
        let async_loader = AsyncLoader::new();

        // Load settings from disk (or use defaults)
        let app_settings = match AppSettings::load() {
            Ok(settings) => {
                tracing::info!("Loaded settings from {:?}", AppSettings::settings_path());
                settings
            }
            Err(e) => {
                tracing::warn!("Failed to load settings, using defaults: {}", e);
                AppSettings::default()
            }
        };

        // Apply saved language setting at startup
        let language = Language::from_code(&app_settings.gui.language);
        i18n::set_language(language);

        // Check if autostart is actually enabled (sync with system state)
        let autostart_enabled = AutostartManager::is_enabled();

        let state = Self {
            current_view: View::Library,
            wallpapers: Vec::new(),
            workshop_wallpapers: Vec::new(),
            local_wallpapers: Vec::new(),
            selected_wallpaper: None,
            folders: Vec::new(),
            search_query: String::new(),
            current_filter: WallpaperFilter::All,
            source_filter: SourceFilter::All,
            loading: false,
            workshop_scanning: false,
            error: None,
            status_message: None,
            daemon_connected: false,
            thumbnails: HashMap::new(),
            thumbnail_states: HashMap::new(),
            pending_thumbnails: HashSet::new(),
            async_loader,
            app_settings: AppSettings {
                autostart: crate::settings::AutostartSettings {
                    enabled: autostart_enabled,
                    ..app_settings.autostart
                },
                ..app_settings
            },
            monitors: Vec::new(),
            workshop_available,
            ipc_state: ConnectionState::Disconnected,
            last_daemon_status: None,
        };

        // Start scanning Workshop on startup
        let task = Task::perform(async {}, |_| Message::ScanWorkshop);

        (state, task)
    }

    /// Update state from daemon status
    pub fn update_from_daemon_status(&mut self, status: DaemonStatus) {
        self.daemon_connected = status.running;
        self.ipc_state = if status.running {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        };

        // Update monitor wallpaper status from daemon
        for output in &status.outputs {
            if let Some(monitor) = self.monitors.iter_mut().find(|m| m.name == output.name) {
                monitor.current_wallpaper = output.wallpaper.as_ref().map(PathBuf::from);
            }
        }

        self.last_daemon_status = Some(status);
    }

    /// Get thumbnail state for a wallpaper
    pub fn get_thumbnail_state(&self, id: &str) -> ThumbnailState {
        if self.thumbnails.contains_key(id) {
            ThumbnailState::Loaded
        } else {
            self.thumbnail_states
                .get(id)
                .cloned()
                .unwrap_or(ThumbnailState::NotLoaded)
        }
    }

    /// Request thumbnail for a wallpaper
    pub fn request_thumbnail(&mut self, wallpaper: &WallpaperItem) -> Option<ThumbnailRequest> {
        let id = &wallpaper.id;

        // Skip if already loaded or loading
        if self.thumbnails.contains_key(id) || self.pending_thumbnails.contains(id) {
            return None;
        }

        // Mark as loading
        self.thumbnail_states
            .insert(id.clone(), ThumbnailState::Loading);
        self.pending_thumbnails.insert(id.clone());

        // Create request - prefer thumbnail_path for Workshop items
        let source_path = wallpaper
            .thumbnail_path
            .clone()
            .unwrap_or_else(|| wallpaper.source_path.clone());

        Some(ThumbnailRequest {
            id: id.clone(),
            path: source_path,
            width: 256,
            height: 144,
        })
    }

    /// Handle thumbnail loaded
    pub fn on_thumbnail_loaded(&mut self, id: String, data: Vec<u8>) {
        self.thumbnails.insert(id.clone(), data);
        self.thumbnail_states
            .insert(id.clone(), ThumbnailState::Loaded);
        self.pending_thumbnails.remove(&id);
    }

    /// Handle thumbnail failed
    pub fn on_thumbnail_failed(&mut self, id: String, error: String) {
        self.thumbnail_states
            .insert(id.clone(), ThumbnailState::Failed(error));
        self.pending_thumbnails.remove(&id);
    }

    /// Get pending thumbnail requests for visible wallpapers
    pub fn get_pending_thumbnail_requests(&self, visible_ids: &[String]) -> Vec<ThumbnailRequest> {
        visible_ids
            .iter()
            .filter_map(|id| {
                // Skip if already loaded or loading
                if self.thumbnails.contains_key(id) || self.pending_thumbnails.contains(id) {
                    return None;
                }

                // Find the wallpaper
                self.wallpapers.iter().find(|wp| &wp.id == id).map(|wp| {
                    let source_path = wp
                        .thumbnail_path
                        .clone()
                        .unwrap_or_else(|| wp.source_path.clone());

                    ThumbnailRequest {
                        id: id.clone(),
                        path: source_path,
                        width: 256,
                        height: 144,
                    }
                })
            })
            .collect()
    }

    /// Merge workshop and local wallpapers into the combined list
    pub fn refresh_wallpapers(&mut self) {
        self.wallpapers.clear();
        self.wallpapers.extend(self.workshop_wallpapers.clone());
        self.wallpapers.extend(self.local_wallpapers.clone());
    }

    /// Get filtered wallpapers based on current search and filter
    pub fn filtered_wallpapers(&self) -> Vec<&WallpaperItem> {
        self.wallpapers
            .iter()
            .filter(|wp| {
                // Apply source filter
                let source_match = match self.source_filter {
                    SourceFilter::All => true,
                    SourceFilter::Workshop => {
                        matches!(wp.source_type, wayvid_core::SourceType::SteamWorkshop)
                    }
                    SourceFilter::Local => {
                        matches!(
                            wp.source_type,
                            wayvid_core::SourceType::LocalFile
                                | wayvid_core::SourceType::LocalDirectory
                        )
                    }
                };

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
                        || wp
                            .metadata
                            .title
                            .as_ref()
                            .map(|t| t.to_lowercase().contains(&query))
                            .unwrap_or(false)
                        || wp
                            .metadata
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&query))
                };

                source_match && type_match && search_match
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
    #[allow(dead_code)] // Reserved for folder toggle feature
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
    /// Get display name for the filter (localized)
    pub fn name(&self) -> String {
        use rust_i18n::t;
        match self {
            Self::All => t!("filter.all").to_string(),
            Self::Videos => t!("filter.videos").to_string(),
            Self::Images => t!("filter.images").to_string(),
            Self::Gifs => t!("filter.gifs").to_string(),
            Self::Scenes => t!("filter.scenes").to_string(),
            Self::Favorites => t!("filter.favorites").to_string(),
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

/// Source filter for wallpapers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceFilter {
    #[default]
    All,
    Workshop,
    Local,
}

impl SourceFilter {
    /// Get display name for the source filter (localized)
    pub fn name(&self) -> String {
        use rust_i18n::t;
        match self {
            Self::All => t!("source.all").to_string(),
            Self::Workshop => t!("source.workshop").to_string(),
            Self::Local => t!("source.local").to_string(),
        }
    }

    /// Get all source filter variants
    pub fn all() -> &'static [SourceFilter] {
        &[Self::All, Self::Workshop, Self::Local]
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

/// Type alias for state used in views
pub type WayvidState = AppState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_state_default() {
        let state = ThumbnailState::NotLoaded;
        assert_eq!(state, ThumbnailState::NotLoaded);
    }

    #[test]
    fn test_thumbnail_state_loaded() {
        let state = ThumbnailState::Loaded;
        assert_eq!(state, ThumbnailState::Loaded);
    }

    #[test]
    fn test_thumbnail_state_failed() {
        let state = ThumbnailState::Failed("error".to_string());
        assert!(matches!(state, ThumbnailState::Failed(_)));
    }

    #[test]
    fn test_get_thumbnail_state_not_loaded() {
        let (state, _) = AppState::new();
        let result = state.get_thumbnail_state("test-id");
        assert_eq!(result, ThumbnailState::NotLoaded);
    }

    #[test]
    fn test_get_thumbnail_state_loaded() {
        let (mut state, _) = AppState::new();
        state
            .thumbnails
            .insert("test-id".to_string(), vec![1, 2, 3]);
        let result = state.get_thumbnail_state("test-id");
        assert_eq!(result, ThumbnailState::Loaded);
    }

    #[test]
    fn test_on_thumbnail_loaded() {
        let (mut state, _) = AppState::new();
        state.pending_thumbnails.insert("test-id".to_string());
        state.on_thumbnail_loaded("test-id".to_string(), vec![1, 2, 3]);

        assert!(state.thumbnails.contains_key("test-id"));
        assert!(!state.pending_thumbnails.contains("test-id"));
        assert_eq!(
            state.thumbnail_states.get("test-id"),
            Some(&ThumbnailState::Loaded)
        );
    }

    #[test]
    fn test_on_thumbnail_failed() {
        let (mut state, _) = AppState::new();
        state.pending_thumbnails.insert("test-id".to_string());
        state.on_thumbnail_failed("test-id".to_string(), "error".to_string());

        assert!(!state.thumbnails.contains_key("test-id"));
        assert!(!state.pending_thumbnails.contains("test-id"));
        assert!(matches!(
            state.thumbnail_states.get("test-id"),
            Some(ThumbnailState::Failed(_))
        ));
    }
}
