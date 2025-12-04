//! Thumbnail image widget with async loading support

use iced::widget::{container, text};
use iced::{Element, Length};

/// Thumbnail loading state
#[derive(Debug, Clone)]
pub enum ThumbnailState {
    NotLoaded,
    Loading,
    Loaded(Vec<u8>),
    Failed(String),
}

impl Default for ThumbnailState {
    fn default() -> Self {
        Self::NotLoaded
    }
}

/// Configuration for thumbnail display
#[derive(Debug, Clone)]
pub struct ThumbnailConfig {
    pub width: f32,
    pub height: f32,
    pub placeholder: String,
    pub show_loading_indicator: bool,
}

impl Default for ThumbnailConfig {
    fn default() -> Self {
        Self {
            width: 200.0,
            height: 120.0,
            placeholder: "🖼️".to_string(),
            show_loading_indicator: true,
        }
    }
}

/// Thumbnail image widget
pub struct ThumbnailImage {
    state: ThumbnailState,
    config: ThumbnailConfig,
}

impl ThumbnailImage {
    pub fn new() -> Self {
        Self {
            state: ThumbnailState::NotLoaded,
            config: ThumbnailConfig::default(),
        }
    }

    pub fn state(mut self, state: ThumbnailState) -> Self {
        self.state = state;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    pub fn view<M: 'static>(self) -> Element<'static, M> {
        let placeholder = self.config.placeholder.clone();
        let width = self.config.width;
        let height = self.config.height;

        let content: Element<'static, M> = match self.state {
            ThumbnailState::NotLoaded => text(placeholder).size(48).into(),
            ThumbnailState::Loading => text("⏳").size(32).into(),
            ThumbnailState::Loaded(_) => text("✓").size(48).into(),
            ThumbnailState::Failed(err) => text(format!("❌\n{}", err)).size(16).into(),
        };

        container(content)
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .center(Length::Fill)
            .style(container::bordered_box)
            .into()
    }
}

impl Default for ThumbnailImage {
    fn default() -> Self {
        Self::new()
    }
}

/// Async thumbnail loader
pub struct ThumbnailLoader {
    cache_dir: std::path::PathBuf,
}

impl ThumbnailLoader {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("wayvid")
            .join("thumbnails");
        Self { cache_dir }
    }

    pub fn cache_dir(&self) -> &std::path::Path {
        &self.cache_dir
    }

    pub fn get_cache_path(&self, wallpaper_id: &str) -> std::path::PathBuf {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        wallpaper_id.hash(&mut hasher);
        let hash = hasher.finish();
        self.cache_dir.join(format!("{:x}.webp", hash))
    }
}

impl Default for ThumbnailLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_state_default() {
        let state = ThumbnailState::default();
        assert!(matches!(state, ThumbnailState::NotLoaded));
    }

    #[test]
    fn test_thumbnail_config_default() {
        let config = ThumbnailConfig::default();
        assert_eq!(config.width, 200.0);
        assert_eq!(config.height, 120.0);
    }

    #[test]
    fn test_thumbnail_loader_cache_path() {
        let loader = ThumbnailLoader::new();
        let path = loader.get_cache_path("test-wallpaper-id");
        assert!(path.to_string_lossy().contains(".webp"));
    }
}
