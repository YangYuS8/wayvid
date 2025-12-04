//! Message types for the application
//!
//! All user interactions and async events are represented as messages.

use std::path::PathBuf;
use wayvid_core::WallpaperItem;

use crate::views::View;
use crate::state::WallpaperFilter;

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    /// Navigate to a different view
    NavigateTo(View),

    // Library operations
    /// Start loading the wallpaper library
    LoadLibrary,
    /// Library loading completed
    LibraryLoaded(Result<Vec<WallpaperItem>, String>),

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

    // Settings
    /// Toggle autostart
    ToggleAutostart(bool),
    /// Toggle minimize to tray
    ToggleMinimizeToTray(bool),
    /// Save settings
    SaveSettings,
    /// Settings saved
    SettingsSaved(Result<(), String>),

    // Theme
    /// Toggle between light and dark theme
    ToggleTheme,

    // Error handling
    /// Dismiss the current error
    DismissError,
    /// Dismiss the current status message
    DismissStatus,

    // Daemon communication
    /// Daemon connected successfully
    DaemonConnected,
    /// Daemon connection lost
    DaemonDisconnected,

    // Thumbnail loading
    /// Thumbnail loaded for a wallpaper
    ThumbnailLoaded(String, Vec<u8>),
}
