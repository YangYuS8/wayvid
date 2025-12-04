//! Folder scanner for discovering wallpapers
//!
//! Scans configured folders for video and image files,
//! creating WallpaperItem entries for the library.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use wayvid_core::{WallpaperItem, WallpaperType, SourceType};

/// File scanner for discovering wallpapers
pub struct FolderScanner {
    /// Supported video extensions
    video_extensions: HashSet<String>,
    /// Supported image extensions  
    image_extensions: HashSet<String>,
}

impl FolderScanner {
    pub fn new() -> Self {
        Self {
            video_extensions: [
                "mp4", "mkv", "webm", "avi", "mov", "m4v", "wmv", "flv"
            ].iter().map(|s| s.to_string()).collect(),
            
            image_extensions: [
                "png", "jpg", "jpeg", "webp", "bmp", "tiff", "tif", "gif"
            ].iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Scan a folder for wallpapers
    pub fn scan_folder(&self, path: &Path, recursive: bool) -> Result<Vec<WallpaperItem>> {
        info!("🔍 Scanning folder: {}", path.display());
        
        if !path.exists() {
            warn!("  ⚠️ Folder does not exist: {}", path.display());
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        
        let walker = if recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();
            
            if !file_path.is_file() {
                continue;
            }

            if let Some(item) = self.process_file(file_path) {
                debug!("  📄 Found: {}", item.name);
                items.push(item);
            }
        }

        info!("  ✓ Found {} wallpapers", items.len());
        Ok(items)
    }

    /// Process a single file and create WallpaperItem if valid
    fn process_file(&self, path: &Path) -> Option<WallpaperItem> {
        let extension = path.extension()?
            .to_string_lossy()
            .to_lowercase();

        let wallpaper_type = if self.video_extensions.contains(&extension) {
            WallpaperType::Video
        } else if extension == "gif" {
            WallpaperType::Gif
        } else if self.image_extensions.contains(&extension) {
            WallpaperType::Image
        } else {
            return None;
        };

        let name = path.file_stem()?
            .to_string_lossy()
            .to_string();

        // Get file metadata
        let file_size = fs::metadata(path).ok().map(|m| m.len());

        let mut item = WallpaperItem::new(
            path.to_owned(),
            name,
            SourceType::LocalFile,
            wallpaper_type,
        );

        // Update metadata with file size
        item.metadata.file_size = file_size;

        Some(item)
    }

    /// Check if a file is a supported wallpaper
    pub fn is_wallpaper_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            self.video_extensions.contains(&ext) || self.image_extensions.contains(&ext)
        } else {
            false
        }
    }

    /// Get the wallpaper type for a file
    pub fn get_wallpaper_type(&self, path: &Path) -> Option<WallpaperType> {
        let ext = path.extension()?.to_string_lossy().to_lowercase();
        
        if self.video_extensions.contains(&ext) {
            Some(WallpaperType::Video)
        } else if ext == "gif" {
            Some(WallpaperType::Gif)
        } else if self.image_extensions.contains(&ext) {
            Some(WallpaperType::Image)
        } else {
            None
        }
    }

    /// Add custom video extension
    pub fn add_video_extension(&mut self, ext: &str) {
        self.video_extensions.insert(ext.to_lowercase());
    }

    /// Add custom image extension
    pub fn add_image_extension(&mut self, ext: &str) {
        self.image_extensions.insert(ext.to_lowercase());
    }
}

impl Default for FolderScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a scan operation
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// New wallpapers found
    pub added: Vec<WallpaperItem>,
    /// Wallpapers that no longer exist (IDs)
    pub removed: Vec<String>,
    /// Wallpapers that were updated
    pub updated: Vec<WallpaperItem>,
    /// Total files scanned
    pub files_scanned: usize,
    /// Scan duration in milliseconds
    pub duration_ms: u64,
}

impl ScanResult {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.updated.is_empty()
    }
}

/// Incremental scanner that tracks changes
pub struct IncrementalScanner {
    scanner: FolderScanner,
    /// Known files and their modification times
    known_files: std::collections::HashMap<PathBuf, SystemTime>,
}

impl IncrementalScanner {
    pub fn new() -> Self {
        Self {
            scanner: FolderScanner::new(),
            known_files: std::collections::HashMap::new(),
        }
    }

    /// Perform incremental scan, only processing changed files
    pub fn scan_incremental(&mut self, path: &Path, recursive: bool) -> Result<ScanResult> {
        let start = std::time::Instant::now();
        let mut result = ScanResult::default();

        if !path.exists() {
            return Ok(result);
        }

        let walker = if recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };

        let mut current_files: HashSet<PathBuf> = HashSet::new();

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();
            
            if !file_path.is_file() || !self.scanner.is_wallpaper_file(file_path) {
                continue;
            }

            result.files_scanned += 1;
            current_files.insert(file_path.to_owned());

            let modified = fs::metadata(file_path)
                .ok()
                .and_then(|m| m.modified().ok());

            if let Some(modified) = modified {
                if let Some(known_modified) = self.known_files.get(file_path) {
                    // Check if file was modified
                    if &modified != known_modified {
                        if let Some(item) = self.scanner.process_file(file_path) {
                            result.updated.push(item);
                        }
                    }
                } else {
                    // New file
                    if let Some(item) = self.scanner.process_file(file_path) {
                        result.added.push(item);
                    }
                }
                self.known_files.insert(file_path.to_owned(), modified);
            }
        }

        // Find removed files
        let removed_paths: Vec<PathBuf> = self.known_files
            .keys()
            .filter(|p| p.starts_with(path) && !current_files.contains(*p))
            .cloned()
            .collect();

        for removed_path in removed_paths {
            let id = WallpaperItem::generate_id(&removed_path);
            result.removed.push(id);
            self.known_files.remove(&removed_path);
        }

        result.duration_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Clear known files cache
    pub fn clear_cache(&mut self) {
        self.known_files.clear();
    }
}

impl Default for IncrementalScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;

    fn create_test_files(dir: &Path) {
        File::create(dir.join("video1.mp4")).unwrap();
        File::create(dir.join("video2.mkv")).unwrap();
        File::create(dir.join("image1.png")).unwrap();
        File::create(dir.join("document.pdf")).unwrap();  // Should be ignored
    }

    #[test]
    fn test_scan_folder() {
        let temp_dir = TempDir::new().unwrap();
        create_test_files(temp_dir.path());

        let scanner = FolderScanner::new();
        let items = scanner.scan_folder(temp_dir.path(), false).unwrap();

        assert_eq!(items.len(), 3);  // 2 videos + 1 image
        
        let video_count = items.iter().filter(|i| i.wallpaper_type == WallpaperType::Video).count();
        let image_count = items.iter().filter(|i| i.wallpaper_type == WallpaperType::Image).count();
        
        assert_eq!(video_count, 2);
        assert_eq!(image_count, 1);
    }

    #[test]
    fn test_is_wallpaper_file() {
        let scanner = FolderScanner::new();
        
        assert!(scanner.is_wallpaper_file(Path::new("test.mp4")));
        assert!(scanner.is_wallpaper_file(Path::new("test.PNG")));  // Case insensitive
        assert!(scanner.is_wallpaper_file(Path::new("test.webm")));
        assert!(!scanner.is_wallpaper_file(Path::new("test.txt")));
        assert!(!scanner.is_wallpaper_file(Path::new("test.pdf")));
    }

    #[test]
    fn test_get_wallpaper_type() {
        let scanner = FolderScanner::new();
        
        assert_eq!(scanner.get_wallpaper_type(Path::new("test.mp4")), Some(WallpaperType::Video));
        assert_eq!(scanner.get_wallpaper_type(Path::new("test.png")), Some(WallpaperType::Image));
        assert_eq!(scanner.get_wallpaper_type(Path::new("test.gif")), Some(WallpaperType::Gif));
        assert_eq!(scanner.get_wallpaper_type(Path::new("test.txt")), None);
    }
}
