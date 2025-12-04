//! SQLite database for wallpaper library persistence
//!
//! Stores wallpaper metadata, thumbnails, and user settings.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tracing::{debug, info};

use wayvid_core::{WallpaperItem, WallpaperMetadata, WallpaperType, SourceType};

/// Wallpaper library database
pub struct LibraryDatabase {
    conn: Arc<RwLock<Connection>>,
    db_path: PathBuf,
}

impl LibraryDatabase {
    /// Open or create database at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        info!("📦 Opening library database: {}", path.display());
        
        let conn = Connection::open(path)
            .context("Failed to open database")?;
        
        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .context("Failed to set database pragmas")?;

        let db = Self {
            conn: Arc::new(RwLock::new(conn)),
            db_path: path.to_owned(),
        };

        db.initialize_schema()?;
        
        Ok(db)
    }

    /// Get default database path
    pub fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wayvid")
            .join("library.db")
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.write().unwrap();
        
        conn.execute_batch(r#"
            -- Wallpaper items table
            CREATE TABLE IF NOT EXISTS wallpapers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source_path TEXT NOT NULL UNIQUE,
                source_type TEXT NOT NULL,
                wallpaper_type TEXT NOT NULL,
                thumbnail_path TEXT,
                
                -- Metadata
                title TEXT,
                author TEXT,
                description TEXT,
                tags TEXT,
                duration_secs REAL,
                resolution_w INTEGER,
                resolution_h INTEGER,
                file_size INTEGER,
                workshop_id INTEGER,
                
                -- Timestamps
                added_at TEXT NOT NULL,
                last_used TEXT,
                
                -- User data
                favorite INTEGER NOT NULL DEFAULT 0,
                use_count INTEGER NOT NULL DEFAULT 0
            );

            -- Tags table for efficient querying
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );

            -- Wallpaper-tag relationship
            CREATE TABLE IF NOT EXISTS wallpaper_tags (
                wallpaper_id TEXT NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (wallpaper_id, tag_id),
                FOREIGN KEY (wallpaper_id) REFERENCES wallpapers(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            -- Library folders
            CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                enabled INTEGER NOT NULL DEFAULT 1,
                scan_recursive INTEGER NOT NULL DEFAULT 1,
                last_scanned_at TEXT
            );

            -- Thumbnails cache
            CREATE TABLE IF NOT EXISTS thumbnails (
                wallpaper_id TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                format TEXT NOT NULL DEFAULT 'png',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (wallpaper_id) REFERENCES wallpapers(id) ON DELETE CASCADE
            );

            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_wallpapers_type ON wallpapers(wallpaper_type);
            CREATE INDEX IF NOT EXISTS idx_wallpapers_source ON wallpapers(source_type);
            CREATE INDEX IF NOT EXISTS idx_wallpapers_favorite ON wallpapers(favorite);
            CREATE INDEX IF NOT EXISTS idx_wallpapers_name ON wallpapers(name);
        "#).context("Failed to initialize database schema")?;

        debug!("  ✓ Database schema initialized");
        Ok(())
    }

    // ========== Wallpaper CRUD ==========

    /// Insert or update a wallpaper
    pub fn upsert_wallpaper(&self, item: &WallpaperItem) -> Result<()> {
        let conn = self.conn.write().unwrap();
        
        let tags_json = serde_json::to_string(&item.metadata.tags).unwrap_or_default();
        let (res_w, res_h) = item.metadata.resolution.unwrap_or((0, 0));
        
        conn.execute(
            r#"
            INSERT INTO wallpapers (
                id, name, source_path, source_type, wallpaper_type, thumbnail_path,
                title, author, description, tags, duration_secs, resolution_w, resolution_h,
                file_size, workshop_id, added_at, last_used
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(source_path) DO UPDATE SET
                name = excluded.name,
                wallpaper_type = excluded.wallpaper_type,
                thumbnail_path = excluded.thumbnail_path,
                title = excluded.title,
                author = excluded.author,
                description = excluded.description,
                tags = excluded.tags,
                duration_secs = excluded.duration_secs,
                resolution_w = excluded.resolution_w,
                resolution_h = excluded.resolution_h,
                file_size = excluded.file_size
            "#,
            params![
                item.id,
                item.name,
                item.source_path.to_string_lossy(),
                item.source_type.as_str(),
                item.wallpaper_type.as_str(),
                item.thumbnail_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                item.metadata.title,
                item.metadata.author,
                item.metadata.description,
                tags_json,
                item.metadata.duration_secs,
                res_w,
                res_h,
                item.metadata.file_size,
                item.metadata.workshop_id,
                item.added_at.to_rfc3339(),
                item.last_used.map(|d| d.to_rfc3339()),
            ],
        )?;
        
        Ok(())
    }

    /// Get wallpaper by ID
    pub fn get_wallpaper(&self, id: &str) -> Result<Option<WallpaperItem>> {
        let conn = self.conn.read().unwrap();
        
        conn.query_row(
            "SELECT * FROM wallpapers WHERE id = ?1",
            params![id],
            |row| self.row_to_wallpaper(row),
        )
        .optional()
        .context("Failed to query wallpaper")
    }

    /// Get wallpaper by path
    pub fn get_wallpaper_by_path(&self, path: &Path) -> Result<Option<WallpaperItem>> {
        let conn = self.conn.read().unwrap();
        
        conn.query_row(
            "SELECT * FROM wallpapers WHERE source_path = ?1",
            params![path.to_string_lossy()],
            |row| self.row_to_wallpaper(row),
        )
        .optional()
        .context("Failed to query wallpaper by path")
    }

    /// List all wallpapers
    pub fn list_wallpapers(&self, filter: &WallpaperFilter) -> Result<Vec<WallpaperItem>> {
        let conn = self.conn.read().unwrap();
        
        let mut sql = String::from("SELECT * FROM wallpapers WHERE 1=1");
        
        if filter.favorites_only {
            sql.push_str(" AND favorite = 1");
        }
        
        if let Some(ref type_filter) = filter.wallpaper_type {
            sql.push_str(&format!(" AND wallpaper_type = '{}'", type_filter.as_str()));
        }
        
        if let Some(ref source_filter) = filter.source_type {
            sql.push_str(&format!(" AND source_type = '{}'", source_filter.as_str()));
        }

        sql.push_str(match filter.sort_by {
            SortBy::Name => " ORDER BY name ASC",
            SortBy::DateAdded => " ORDER BY added_at DESC",
            SortBy::LastUsed => " ORDER BY last_used DESC NULLS LAST",
            SortBy::UseCount => " ORDER BY use_count DESC",
        });

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut stmt = conn.prepare(&sql)?;
        let wallpapers = stmt
            .query_map([], |row| self.row_to_wallpaper(row))?
            .filter_map(|r| r.ok())
            .collect();
        
        Ok(wallpapers)
    }

    /// Delete wallpaper by ID
    pub fn delete_wallpaper(&self, id: &str) -> Result<bool> {
        let conn = self.conn.write().unwrap();
        let rows = conn.execute("DELETE FROM wallpapers WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        let conn = self.conn.write().unwrap();
        conn.execute(
            "UPDATE wallpapers SET favorite = NOT favorite WHERE id = ?1",
            params![id],
        )?;
        
        let new_state: bool = conn.query_row(
            "SELECT favorite FROM wallpapers WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        
        Ok(new_state)
    }

    /// Record wallpaper usage
    pub fn record_usage(&self, id: &str) -> Result<()> {
        let conn = self.conn.write().unwrap();
        conn.execute(
            "UPDATE wallpapers SET use_count = use_count + 1, last_used = ?2 WHERE id = ?1",
            params![id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    // ========== Folders ==========

    /// Add a library folder
    pub fn add_folder(&self, path: &Path, recursive: bool) -> Result<()> {
        let conn = self.conn.write().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO folders (path, scan_recursive) VALUES (?1, ?2)",
            params![path.to_string_lossy(), recursive as i32],
        )?;
        Ok(())
    }

    /// List all library folders
    pub fn list_folders(&self) -> Result<Vec<LibraryFolder>> {
        let conn = self.conn.read().unwrap();
        let mut stmt = conn.prepare(
            "SELECT path, enabled, scan_recursive, last_scanned_at FROM folders"
        )?;
        
        let folders = stmt
            .query_map([], |row| {
                Ok(LibraryFolder {
                    path: PathBuf::from(row.get::<_, String>(0)?),
                    enabled: row.get::<_, i32>(1)? != 0,
                    recursive: row.get::<_, i32>(2)? != 0,
                    last_scanned: row.get::<_, Option<String>>(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        
        Ok(folders)
    }

    /// Remove a library folder
    pub fn remove_folder(&self, path: &Path) -> Result<bool> {
        let conn = self.conn.write().unwrap();
        let rows = conn.execute(
            "DELETE FROM folders WHERE path = ?1",
            params![path.to_string_lossy()],
        )?;
        Ok(rows > 0)
    }

    // ========== Thumbnails ==========

    /// Store thumbnail data
    pub fn store_thumbnail(&self, wallpaper_id: &str, data: &[u8], width: u32, height: u32) -> Result<()> {
        let conn = self.conn.write().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO thumbnails (wallpaper_id, data, width, height) VALUES (?1, ?2, ?3, ?4)",
            params![wallpaper_id, data, width, height],
        )?;
        Ok(())
    }

    /// Get thumbnail data
    pub fn get_thumbnail(&self, wallpaper_id: &str) -> Result<Option<ThumbnailData>> {
        let conn = self.conn.read().unwrap();
        conn.query_row(
            "SELECT data, width, height FROM thumbnails WHERE wallpaper_id = ?1",
            params![wallpaper_id],
            |row| {
                Ok(ThumbnailData {
                    data: row.get(0)?,
                    width: row.get(1)?,
                    height: row.get(2)?,
                })
            },
        )
        .optional()
        .context("Failed to query thumbnail")
    }

    // ========== Stats ==========

    /// Get library statistics
    pub fn get_stats(&self) -> Result<LibraryStats> {
        let conn = self.conn.read().unwrap();
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM wallpapers", [], |row| row.get(0)
        )?;
        
        let favorites: i64 = conn.query_row(
            "SELECT COUNT(*) FROM wallpapers WHERE favorite = 1", [], |row| row.get(0)
        )?;
        
        let videos: i64 = conn.query_row(
            "SELECT COUNT(*) FROM wallpapers WHERE wallpaper_type = 'video'", [], |row| row.get(0)
        )?;
        
        let images: i64 = conn.query_row(
            "SELECT COUNT(*) FROM wallpapers WHERE wallpaper_type = 'image'", [], |row| row.get(0)
        )?;
        
        let total_size: i64 = conn.query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM wallpapers", [], |row| row.get(0)
        )?;

        Ok(LibraryStats {
            total_wallpapers: total as usize,
            favorites: favorites as usize,
            videos: videos as usize,
            images: images as usize,
            total_size_bytes: total_size as u64,
        })
    }

    // ========== Helpers ==========

    fn row_to_wallpaper(&self, row: &rusqlite::Row) -> rusqlite::Result<WallpaperItem> {
        let id: String = row.get("id")?;
        let name: String = row.get("name")?;
        let source_path: String = row.get("source_path")?;
        let source_type_str: String = row.get("source_type")?;
        let wallpaper_type_str: String = row.get("wallpaper_type")?;
        let thumbnail_path: Option<String> = row.get("thumbnail_path")?;
        
        let title: Option<String> = row.get("title")?;
        let author: Option<String> = row.get("author")?;
        let description: Option<String> = row.get("description")?;
        let tags_json: Option<String> = row.get("tags")?;
        let duration_secs: Option<f64> = row.get("duration_secs")?;
        let res_w: Option<u32> = row.get("resolution_w")?;
        let res_h: Option<u32> = row.get("resolution_h")?;
        let file_size: Option<u64> = row.get("file_size")?;
        let workshop_id: Option<u64> = row.get("workshop_id")?;
        let added_at_str: String = row.get("added_at")?;
        let last_used_str: Option<String> = row.get("last_used")?;

        let tags: Vec<String> = tags_json
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default();

        let resolution = match (res_w, res_h) {
            (Some(w), Some(h)) if w > 0 && h > 0 => Some((w, h)),
            _ => None,
        };

        let metadata = WallpaperMetadata {
            title,
            author,
            description,
            tags,
            duration_secs,
            resolution,
            file_size,
            workshop_id,
        };

        let added_at = DateTime::parse_from_rfc3339(&added_at_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let last_used = last_used_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|d| d.with_timezone(&Utc))
                .ok()
        });

        Ok(WallpaperItem {
            id,
            name,
            source_path: PathBuf::from(source_path),
            source_type: str_to_source_type(&source_type_str),
            wallpaper_type: str_to_wallpaper_type(&wallpaper_type_str),
            thumbnail_path: thumbnail_path.map(PathBuf::from),
            metadata,
            added_at,
            last_used,
        })
    }
}

// ========== Types ==========

/// Filter options for wallpaper queries
#[derive(Debug, Clone, Default)]
pub struct WallpaperFilter {
    pub favorites_only: bool,
    pub wallpaper_type: Option<WallpaperType>,
    pub source_type: Option<SourceType>,
    pub sort_by: SortBy,
    pub limit: Option<usize>,
}

/// Sort order for wallpaper queries
#[derive(Debug, Clone, Copy, Default)]
pub enum SortBy {
    #[default]
    Name,
    DateAdded,
    LastUsed,
    UseCount,
}

/// Library folder info
#[derive(Debug, Clone)]
pub struct LibraryFolder {
    pub path: PathBuf,
    pub enabled: bool,
    pub recursive: bool,
    pub last_scanned: Option<String>,
}

/// Thumbnail data
#[derive(Debug, Clone)]
pub struct ThumbnailData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Library statistics
#[derive(Debug, Clone, Default)]
pub struct LibraryStats {
    pub total_wallpapers: usize,
    pub favorites: usize,
    pub videos: usize,
    pub images: usize,
    pub total_size_bytes: u64,
}

// ========== Conversion helpers ==========

fn str_to_wallpaper_type(s: &str) -> WallpaperType {
    match s {
        "video" => WallpaperType::Video,
        "image" => WallpaperType::Image,
        "scene" => WallpaperType::Scene,
        "gif" => WallpaperType::Gif,
        _ => WallpaperType::Video,
    }
}

fn str_to_source_type(s: &str) -> SourceType {
    match s {
        "local_file" => SourceType::LocalFile,
        "local_dir" => SourceType::LocalDirectory,
        "workshop" => SourceType::SteamWorkshop,
        _ => SourceType::LocalFile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (LibraryDatabase, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = LibraryDatabase::open(&db_path).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_database_creation() {
        let (db, _temp) = create_test_db();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_wallpapers, 0);
    }

    #[test]
    fn test_wallpaper_crud() {
        let (db, _temp) = create_test_db();

        let item = WallpaperItem::new(
            PathBuf::from("/tmp/test.mp4"),
            "Test Video".to_string(),
            SourceType::LocalFile,
            WallpaperType::Video,
        );

        // Insert
        db.upsert_wallpaper(&item).unwrap();

        // Query
        let retrieved = db.get_wallpaper(&item.id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test Video");
        assert_eq!(retrieved.wallpaper_type, WallpaperType::Video);

        // Toggle favorite
        let is_fav = db.toggle_favorite(&item.id).unwrap();
        assert!(is_fav);

        // Delete
        let deleted = db.delete_wallpaper(&item.id).unwrap();
        assert!(deleted);

        // Verify deleted
        let none = db.get_wallpaper(&item.id).unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_library_folders() {
        let (db, _temp) = create_test_db();

        db.add_folder(Path::new("/home/user/wallpapers"), true).unwrap();
        db.add_folder(Path::new("/home/user/videos"), false).unwrap();

        let folders = db.list_folders().unwrap();
        assert_eq!(folders.len(), 2);
        assert!(folders.iter().any(|f| f.path.to_string_lossy().contains("wallpapers")));

        db.remove_folder(Path::new("/home/user/wallpapers")).unwrap();
        let folders = db.list_folders().unwrap();
        assert_eq!(folders.len(), 1);
    }
}
