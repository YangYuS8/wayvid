//! Workshop item management

use crate::we::parser::parse_we_project;
use crate::we::types::WeProject;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub const WALLPAPER_ENGINE_APP_ID: u32 = 431960;

/// Workshop item metadata
#[derive(Debug, Clone)]
pub struct WorkshopItem {
    /// Workshop item ID
    pub id: u64,
    /// Path to item directory
    pub path: PathBuf,
    /// Parsed WE project (if valid)
    pub project: Option<WeProject>,
}

impl WorkshopItem {
    /// Load from directory
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let id = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|s| s.parse().ok())
            .context("Invalid workshop item ID")?;

        // Parser expects project.json file path
        let project_json = path.join("project.json");
        let project = parse_we_project(&project_json).ok().map(|(proj, _)| proj);

        Ok(Self { id, path, project })
    }

    /// Get item title
    pub fn title(&self) -> String {
        self.project
            .as_ref()
            .and_then(|p| p.title.clone())
            .unwrap_or_else(|| format!("Workshop Item {}", self.id))
    }

    /// Check if valid video wallpaper
    pub fn is_valid(&self) -> bool {
        self.project.is_some()
    }

    /// Get video file path
    pub fn video_path(&self) -> Option<PathBuf> {
        self.project
            .as_ref()?
            .file
            .as_ref()
            .map(|f| self.path.join(f))
    }

    /// Get preview image path (gif or jpg)
    #[allow(dead_code)]
    pub fn preview_path(&self) -> Option<PathBuf> {
        // First check project.json preview field
        if let Some(ref project) = self.project {
            if let Some(ref preview) = project.preview {
                let preview_path = self.path.join(preview);
                if preview_path.exists() {
                    return Some(preview_path);
                }
            }
        }

        // Fallback: check for common preview file names
        let preview_names = ["preview.gif", "preview.jpg", "preview.png", "thumb.jpg"];
        for name in &preview_names {
            let path = self.path.join(name);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Get wallpaper type (video, scene, web, etc.)
    #[allow(dead_code)]
    pub fn wallpaper_type(&self) -> WallpaperEngineType {
        self.project
            .as_ref()
            .map(|p| match p.project_type.to_lowercase().as_str() {
                "video" => WallpaperEngineType::Video,
                "scene" => WallpaperEngineType::Scene,
                "web" => WallpaperEngineType::Web,
                "application" => WallpaperEngineType::Application,
                _ => WallpaperEngineType::Unknown,
            })
            .unwrap_or(WallpaperEngineType::Unknown)
    }

    /// Check if this is a scene wallpaper
    #[allow(dead_code)]
    pub fn is_scene(&self) -> bool {
        matches!(self.wallpaper_type(), WallpaperEngineType::Scene)
    }

    /// Check if this is a video wallpaper
    #[allow(dead_code)]
    pub fn is_video(&self) -> bool {
        matches!(self.wallpaper_type(), WallpaperEngineType::Video)
    }
}

/// Wallpaper Engine wallpaper types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum WallpaperEngineType {
    Video,
    Scene,
    Web,
    Application,
    Unknown,
}

/// Workshop scanner
pub struct WorkshopScanner {
    items: Vec<WorkshopItem>,
}

impl WorkshopScanner {
    /// Scan workshop directories
    pub fn scan(paths: &[PathBuf]) -> Result<Self> {
        let mut items = Vec::new();

        for path in paths {
            if let Ok(item) = WorkshopItem::from_path(path.clone()) {
                if item.is_valid() {
                    items.push(item);
                }
            }
        }

        tracing::info!("Loaded {} valid workshop items", items.len());
        Ok(Self { items })
    }

    /// Get all items
    pub fn items(&self) -> &[WorkshopItem] {
        &self.items
    }

    /// Find item by ID
    pub fn find(&self, id: u64) -> Option<&WorkshopItem> {
        self.items.iter().find(|item| item.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workshop_item_title() {
        let item = WorkshopItem {
            id: 123456,
            path: PathBuf::from("/test"),
            project: None,
        };
        assert_eq!(item.title(), "Workshop Item 123456");
    }
}
