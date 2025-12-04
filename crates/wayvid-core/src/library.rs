//! Wallpaper library types
//!
//! Core types for the wallpaper library system (used by wayvid-library crate).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Wallpaper item in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperItem {
    /// Unique identifier (SHA256 of source_path)
    pub id: String,

    /// Display name
    pub name: String,

    /// Source file/directory path
    pub source_path: PathBuf,

    /// Source type (local, workshop, etc.)
    pub source_type: SourceType,

    /// Wallpaper content type
    pub wallpaper_type: WallpaperType,

    /// Path to cached thumbnail (if generated)
    pub thumbnail_path: Option<PathBuf>,

    /// Additional metadata
    pub metadata: WallpaperMetadata,

    /// When this wallpaper was added to library
    pub added_at: DateTime<Utc>,

    /// When this wallpaper was last used
    pub last_used: Option<DateTime<Utc>>,
}

impl WallpaperItem {
    /// Generate a unique ID from the source path
    pub fn generate_id(path: &PathBuf) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string() // Use first 16 hex chars
    }

    /// Create a new wallpaper item
    pub fn new(
        source_path: PathBuf,
        name: String,
        source_type: SourceType,
        wallpaper_type: WallpaperType,
    ) -> Self {
        let id = Self::generate_id(&source_path);
        Self {
            id,
            name,
            source_path,
            source_type,
            wallpaper_type,
            thumbnail_path: None,
            metadata: WallpaperMetadata::default(),
            added_at: Utc::now(),
            last_used: None,
        }
    }
}

/// Source type of a wallpaper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Local file on disk
    LocalFile,

    /// Local directory containing videos
    LocalDirectory,

    /// Steam Workshop item
    SteamWorkshop,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::LocalFile => "local_file",
            SourceType::LocalDirectory => "local_dir",
            SourceType::SteamWorkshop => "workshop",
        }
    }
}

/// Wallpaper content type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperType {
    /// Video file (mp4, mkv, webm, etc.)
    Video,

    /// Wallpaper Engine scene (JSON-based)
    Scene,

    /// Animated GIF
    Gif,

    /// Static image (for future support)
    Image,
}

impl WallpaperType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WallpaperType::Video => "video",
            WallpaperType::Scene => "scene",
            WallpaperType::Gif => "gif",
            WallpaperType::Image => "image",
        }
    }

    /// Get icon for UI display
    pub fn icon(&self) -> &'static str {
        match self {
            WallpaperType::Video => "🎬",
            WallpaperType::Scene => "🎨",
            WallpaperType::Gif => "🖼️",
            WallpaperType::Image => "📷",
        }
    }
}

/// Additional metadata for a wallpaper
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WallpaperMetadata {
    /// Title (from project.json or filename)
    pub title: Option<String>,

    /// Author (from project.json)
    pub author: Option<String>,

    /// Description
    pub description: Option<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Video duration in seconds
    pub duration_secs: Option<f64>,

    /// Resolution (width, height)
    pub resolution: Option<(u32, u32)>,

    /// File size in bytes
    pub file_size: Option<u64>,

    /// Steam Workshop ID (if applicable)
    pub workshop_id: Option<u64>,
}

/// Filter criteria for querying wallpapers
#[derive(Debug, Clone, Default)]
pub struct WallpaperFilter {
    /// Filter by wallpaper type
    pub wallpaper_type: Option<WallpaperType>,

    /// Filter by source type
    pub source_type: Option<SourceType>,

    /// Filter by folder path
    pub folder: Option<PathBuf>,

    /// Search query (matches name, title, tags)
    pub search: Option<String>,

    /// Maximum results to return
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Scan result summary
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// Number of new wallpapers found
    pub added: usize,

    /// Number of wallpapers updated
    pub updated: usize,

    /// Number of wallpapers removed (files deleted)
    pub removed: usize,

    /// Number of errors encountered
    pub errors: usize,

    /// Total wallpapers after scan
    pub total: usize,
}

/// Folder information in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    /// Folder path
    pub path: PathBuf,

    /// Whether folder is enabled for scanning
    pub enabled: bool,

    /// Last scan timestamp
    pub last_scan: Option<DateTime<Utc>>,

    /// Number of wallpapers in this folder
    pub wallpaper_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let path = PathBuf::from("/home/user/videos/wallpaper.mp4");
        let id = WallpaperItem::generate_id(&path);
        assert_eq!(id.len(), 16);

        // Same path should generate same ID
        let id2 = WallpaperItem::generate_id(&path);
        assert_eq!(id, id2);

        // Different path should generate different ID
        let path2 = PathBuf::from("/home/user/videos/other.mp4");
        let id3 = WallpaperItem::generate_id(&path2);
        assert_ne!(id, id3);
    }

    #[test]
    fn test_wallpaper_type_icon() {
        assert_eq!(WallpaperType::Video.icon(), "🎬");
        assert_eq!(WallpaperType::Scene.icon(), "🎨");
    }
}
