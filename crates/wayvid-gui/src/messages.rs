//! Message types for the application
//!
//! All user interactions and async events are represented as messages.

use std::path::PathBuf;
use wayvid_core::WallpaperItem;

use crate::async_loader::ThumbnailRequest;
use crate::i18n::Language;
use crate::ipc::{ConnectionState, DaemonStatus};
use crate::state::{SourceFilter, WallpaperFilter};
use crate::views::View;

/// Application messages
///
/// Some variants are reserved for features under development.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants reserved for future features
pub enum Message {
    // Navigation
    /// Navigate to a different view
    NavigateTo(View),

    // Library operations
    /// Start loading the wallpaper library
    LoadLibrary,
    /// Library loading completed
    LibraryLoaded(Result<Vec<WallpaperItem>, String>),

    // Workshop operations
    /// Start scanning Steam Workshop
    ScanWorkshop,
    /// Workshop scan completed
    WorkshopScanned(Result<Vec<WallpaperItem>, String>),

    // Wallpaper selection
    /// Select a wallpaper (single click)
    SelectWallpaper(String),
    /// Apply wallpaper (double click)
    ApplyWallpaper(String),
    /// Wallpaper application completed
    WallpaperApplied(Result<(), String>),

    // Folder management
    /// Add a new folder to scan
    AddFolder,
    /// Remove a folder from the library
    RemoveFolder(PathBuf),
    /// Start scanning a folder
    ScanFolder(PathBuf),
    /// Folder scan completed
    FolderScanned(Result<Vec<WallpaperItem>, String>),

    // Search and filter
    /// Search query changed
    SearchChanged(String),
    /// Filter changed
    FilterChanged(WallpaperFilter),
    /// Source filter changed
    SourceFilterChanged(SourceFilter),

    // Settings
    /// Toggle autostart
    ToggleAutostart(bool),
    /// Toggle minimize to tray
    ToggleMinimizeToTray(bool),
    /// Toggle pause on battery
    TogglePauseOnBattery(bool),
    /// Toggle pause on fullscreen
    TogglePauseOnFullscreen(bool),
    /// Change volume
    VolumeChanged(f32),
    /// Change FPS limit
    FpsLimitChanged(Option<u32>),
    /// Change language
    LanguageChanged(Language),
    /// Save settings
    SaveSettings,
    /// Settings saved
    SettingsSaved(Result<(), String>),

    // Theme
    /// Toggle between light and dark theme
    ToggleTheme,

    // Layout
    /// Toggle sidebar collapsed state
    ToggleSidebar,
    /// Toggle detail panel visibility
    ToggleDetailPanel,

    // Renderer
    /// Change renderer backend (requires restart)
    ChangeRenderer(String),

    // Error handling
    /// Dismiss the current error
    DismissError,
    /// Dismiss the current status message
    DismissStatus,

    // Window events
    /// Window close requested
    WindowCloseRequested,

    // Daemon communication
    /// Daemon connected successfully
    DaemonConnected,
    /// Daemon connection lost
    DaemonDisconnected,

    // Thumbnail loading
    /// Request thumbnails for a batch of wallpapers
    RequestThumbnails(Vec<ThumbnailRequest>),
    /// Thumbnail loaded for a wallpaper
    ThumbnailLoaded(String, Vec<u8>),
    /// Thumbnail loading failed
    ThumbnailFailed(String, String),
    /// Thumbnail batch complete
    ThumbnailBatchComplete(usize),

    // Monitor operations
    /// Refresh monitor list
    RefreshMonitors,
    /// Monitor list updated
    MonitorsUpdated(Vec<crate::state::MonitorInfo>),
    /// Select a monitor for wallpaper application
    SelectMonitor(String),
    /// Apply wallpaper to a specific monitor
    ApplyToMonitor(String),
    /// Clear wallpaper from a specific monitor
    ClearMonitor(String),

    // Engine control
    /// Start the playback engine
    StartEngine,
    /// Stop the playback engine
    StopEngine,
    /// Engine status updated
    EngineStatusUpdated(bool),
    /// Poll for engine events (timer-driven)
    PollEngineEvents,
    /// Engine event received
    EngineEvent(wayvid_engine::engine::EngineEvent),

    // IPC communication
    /// IPC connection state changed
    IpcConnectionChanged(ConnectionState),
    /// IPC status received from daemon
    IpcStatusReceived(DaemonStatus),
    /// IPC error occurred
    IpcError(String),
}
