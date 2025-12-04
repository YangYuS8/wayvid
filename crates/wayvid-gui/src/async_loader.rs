//! Async operations for wayvid-gui
//!
//! This module provides asynchronous loading of thumbnails and library data,
//! including disk caching and memory management for efficient wallpaper browsing.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use iced::Subscription;
use tokio::sync::RwLock;
use wayvid_core::WallpaperItem;

/// Thumbnail load request
#[derive(Debug, Clone)]
pub struct ThumbnailRequest {
    pub id: String,
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// Thumbnail load result
#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    pub id: String,
    pub result: Result<Vec<u8>, String>,
}

/// Async loader state
#[derive(Debug)]
pub struct AsyncLoader {
    pending_thumbnails: Arc<RwLock<Vec<ThumbnailRequest>>>,
    thumbnail_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    cache_dir: PathBuf,
}

impl AsyncLoader {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("wayvid")
            .join("thumbnails");

        // Ensure cache directory exists
        if !cache_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&cache_dir) {
                tracing::warn!("Failed to create thumbnail cache directory: {}", e);
            }
        }

        Self {
            pending_thumbnails: Arc::new(RwLock::new(Vec::new())),
            thumbnail_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_dir,
        }
    }

    pub async fn request_thumbnail(&self, request: ThumbnailRequest) {
        let mut pending = self.pending_thumbnails.write().await;
        if !pending.iter().any(|r| r.id == request.id) {
            pending.push(request);
        }
    }

    pub async fn get_cached(&self, id: &str) -> Option<Vec<u8>> {
        let cache = self.thumbnail_cache.read().await;
        cache.get(id).cloned()
    }

    pub async fn cache_thumbnail(&self, id: String, data: Vec<u8>) {
        let mut cache = self.thumbnail_cache.write().await;
        cache.insert(id, data);
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}

impl Default for AsyncLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Event from thumbnail loader subscription
#[derive(Debug, Clone)]
pub enum LoaderEvent {
    ThumbnailLoaded(String, Vec<u8>),
    ThumbnailFailed(String, String),
    LibraryLoaded(Result<Vec<WallpaperItem>, String>),
    BatchComplete(usize),
}

/// Create a subscription for thumbnail loading
pub fn thumbnail_subscription<M>(
    requests: Vec<ThumbnailRequest>,
    cache_dir: PathBuf,
    on_event: impl Fn(LoaderEvent) -> M + Send + Sync + Clone + 'static,
) -> Subscription<M>
where
    M: 'static + Send,
{
    use iced::futures::stream;

    Subscription::run_with_id(
        "thumbnail_loader",
        stream::unfold(
            (requests, cache_dir, 0),
            move |(requests, cache_dir, idx)| {
                let on_event = on_event.clone();
                async move {
                    if idx >= requests.len() {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                        return Some((on_event(LoaderEvent::BatchComplete(idx)), (requests, cache_dir, idx)));
                    }

                    let request = &requests[idx];
                    let result = load_thumbnail(request, &cache_dir).await;
                    let event = match result {
                        Ok(data) => LoaderEvent::ThumbnailLoaded(request.id.clone(), data),
                        Err(e) => LoaderEvent::ThumbnailFailed(request.id.clone(), e),
                    };

                    Some((on_event(event), (requests, cache_dir, idx + 1)))
                }
            },
        ),
    )
}

/// Load a single thumbnail
async fn load_thumbnail(request: &ThumbnailRequest, cache_dir: &PathBuf) -> Result<Vec<u8>, String> {
    let cache_path = get_cache_path(cache_dir, &request.id);
    if cache_path.exists() {
        return tokio::fs::read(&cache_path)
            .await
            .map_err(|e| format!("Failed to read cache: {}", e));
    }

    let path = request.path.clone();
    let width = request.width;
    let height = request.height;

    let result = tokio::task::spawn_blocking(move || {
        let generator = wayvid_library::ThumbnailGenerator::with_size(width, height);
        generator.generate(&path)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Thumbnail generation failed: {}", e))?;

    let _ = tokio::fs::write(&cache_path, &result.data).await;

    Ok(result.data)
}

/// Get cache path for a wallpaper ID
fn get_cache_path(cache_dir: &PathBuf, id: &str) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    let hash = hasher.finish();
    cache_dir.join(format!("{:x}.webp", hash))
}

/// Library loading subscription
pub fn library_subscription<M>(
    db_path: PathBuf,
    on_loaded: impl Fn(Result<Vec<WallpaperItem>, String>) -> M + Send + Sync + Clone + 'static,
) -> Subscription<M>
where
    M: 'static + Send,
{
    use iced::futures::stream;

    Subscription::run_with_id(
        "library_loader",
        stream::unfold((db_path, false), move |(db_path, done)| {
            let on_loaded = on_loaded.clone();
            async move {
                if done {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    return None;
                }

                let result = load_library(&db_path).await;
                Some((on_loaded(result), (db_path, true)))
            }
        }),
    )
}

/// Load library from database
async fn load_library(db_path: &PathBuf) -> Result<Vec<WallpaperItem>, String> {
    let path = db_path.clone();

    tokio::task::spawn_blocking(move || {
        let db = wayvid_library::LibraryDatabase::open(&path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let filter = wayvid_library::WallpaperFilter::default();
        db.list_wallpapers(&filter)
            .map_err(|e| format!("Failed to load wallpapers: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Virtual scrolling helper
#[derive(Debug, Clone)]
pub struct VirtualScroll {
    pub total_items: usize,
    pub items_per_row: usize,
    pub row_height: f32,
    pub scroll_offset: f32,
    pub viewport_height: f32,
    pub buffer_rows: usize,
}

impl VirtualScroll {
    pub fn new(total_items: usize, items_per_row: usize, row_height: f32) -> Self {
        Self {
            total_items,
            items_per_row,
            row_height,
            scroll_offset: 0.0,
            viewport_height: 600.0,
            buffer_rows: 2,
        }
    }

    pub fn set_scroll(&mut self, offset: f32) {
        self.scroll_offset = offset.max(0.0);
    }

    pub fn total_height(&self) -> f32 {
        let total_rows = (self.total_items + self.items_per_row - 1) / self.items_per_row;
        total_rows as f32 * self.row_height
    }

    pub fn visible_range(&self) -> (usize, usize) {
        let first_visible_row = (self.scroll_offset / self.row_height) as usize;
        let visible_rows = (self.viewport_height / self.row_height).ceil() as usize;

        let start_row = first_visible_row.saturating_sub(self.buffer_rows);
        let end_row = (first_visible_row + visible_rows + self.buffer_rows)
            .min((self.total_items + self.items_per_row - 1) / self.items_per_row);

        let start_index = start_row * self.items_per_row;
        let end_index = (end_row * self.items_per_row).min(self.total_items);

        (start_index, end_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_scroll_visible_range() {
        let mut scroll = VirtualScroll::new(100, 4, 150.0);
        scroll.viewport_height = 600.0;
        scroll.set_scroll(0.0);
        let (start, end) = scroll.visible_range();
        assert_eq!(start, 0);
        assert!(end > 16);
    }

    #[test]
    fn test_virtual_scroll_total_height() {
        let scroll = VirtualScroll::new(100, 4, 150.0);
        assert_eq!(scroll.total_height(), 3750.0);
    }

    #[test]
    fn test_cache_path() {
        let cache_dir = PathBuf::from("/tmp/test");
        let path = get_cache_path(&cache_dir, "test-id");
        assert!(path.to_string_lossy().ends_with(".webp"));
    }
}
