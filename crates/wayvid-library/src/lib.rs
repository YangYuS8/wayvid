//! wayvid-library: Wallpaper library management for wayvid
//!
//! This crate provides wallpaper library functionality:
//! - SQLite database for wallpaper indexing and metadata
//! - Folder scanning and change detection
//! - Thumbnail generation and caching
//! - Library statistics and queries
//!
//! # Example
//!
//! ```no_run
//! use wayvid_library::{LibraryDatabase, FolderScanner, ThumbnailGenerator};
//! use std::path::Path;
//!
//! // Open or create the library database
//! let db = LibraryDatabase::open(LibraryDatabase::default_path()).unwrap();
//!
//! // Scan a folder for wallpapers
//! let scanner = FolderScanner::new();
//! let wallpapers = scanner.scan_folder(Path::new("/home/user/wallpapers"), true).unwrap();
//!
//! // Add wallpapers to database
//! for wallpaper in &wallpapers {
//!     db.upsert_wallpaper(wallpaper).unwrap();
//! }
//!
//! // Generate thumbnails
//! let thumbgen = ThumbnailGenerator::new();
//! for wallpaper in &wallpapers {
//!     if let Ok(thumb) = thumbgen.generate(&wallpaper.source_path) {
//!         db.store_thumbnail(&wallpaper.id, &thumb.data, thumb.width, thumb.height).unwrap();
//!     }
//! }
//! ```

pub mod database;
pub mod scanner;
pub mod thumbnail;
pub mod workshop;

// Re-exports
pub use database::{
    Collection, LibraryDatabase, LibraryFolder, LibraryStats, SearchOptions, SortBy, Tag,
    ThumbnailData, WallpaperFilter,
};
pub use scanner::{AsyncFileWatcher, FileEvent, FileWatcher, FolderScanner, IncrementalScanner, ScanResult};
pub use thumbnail::{
    get_video_dimensions, get_video_duration, CacheStats, ThumbnailFormat, ThumbnailGenerator,
    ThumbnailPriority, ThumbnailRequest, ThumbnailResponse, ThumbnailResult, ThumbnailService,
};

// Re-exports from wayvid-core
pub use wayvid_core::{SourceType, WallpaperItem, WallpaperMetadata, WallpaperType};

// Workshop exports
pub use workshop::{
    get_project_type, is_we_project, SteamLibrary, WeProject, WorkshopScanner,
    WALLPAPER_ENGINE_APP_ID,
};
