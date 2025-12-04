//! Wallpaper card widget
//!
//! A card component that displays a wallpaper thumbnail with metadata.
//! This component will replace the inline card implementation.

#![allow(dead_code)]

use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length};
use wayvid_core::{WallpaperItem, WallpaperType};

/// Message types for WallpaperCard interactions
#[derive(Debug, Clone)]
pub enum CardMessage {
    Selected(String),
    Applied(String),
    ToggleFavorite(String),
}

/// Configuration for WallpaperCard appearance
#[derive(Debug, Clone)]
pub struct CardConfig {
    pub width: f32,
    pub thumbnail_height: f32,
    pub show_type_badge: bool,
    pub show_favorite: bool,
    pub compact: bool,
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            width: 200.0,
            thumbnail_height: 120.0,
            show_type_badge: true,
            show_favorite: true,
            compact: false,
        }
    }
}

/// Wallpaper card widget
pub struct WallpaperCard<'a> {
    wallpaper: &'a WallpaperItem,
    thumbnail: Option<&'a [u8]>,
    is_selected: bool,
    is_favorite: bool,
    config: CardConfig,
}

impl<'a> WallpaperCard<'a> {
    pub fn new(wallpaper: &'a WallpaperItem) -> Self {
        Self {
            wallpaper,
            thumbnail: None,
            is_selected: false,
            is_favorite: false,
            config: CardConfig::default(),
        }
    }

    pub fn thumbnail(mut self, data: Option<&'a [u8]>) -> Self {
        self.thumbnail = data;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn view<F>(self, on_message: F) -> Element<'a, CardMessage>
    where
        F: 'a + Fn(CardMessage) -> CardMessage,
    {
        let id = self.wallpaper.id.clone();
        let icon = match self.wallpaper.wallpaper_type {
            WallpaperType::Video => "🎬",
            WallpaperType::Image => "🖼️",
            WallpaperType::Gif => "🎞️",
            WallpaperType::Scene => "🎮",
        };

        let thumbnail_display: Element<'_, CardMessage> = container(text(icon).size(48))
            .width(Length::Fill)
            .height(Length::Fixed(self.config.thumbnail_height))
            .center_x(Length::Fill)
            .center_y(Length::Fixed(self.config.thumbnail_height))
            .style(container::bordered_box)
            .into();

        let title = text(&self.wallpaper.name).size(14).width(Length::Fill);
        let type_badge = text(format!("{:?}", self.wallpaper.wallpaper_type)).size(10);

        let resolution = if let Some((w, h)) = self.wallpaper.metadata.resolution {
            Some(text(format!("{}×{}", w, h)).size(10))
        } else {
            None
        };

        let mut info_row = row![type_badge].spacing(8);
        if let Some(res) = resolution {
            info_row = info_row.push(res);
        }
        if self.config.show_favorite {
            let fav_icon = if self.is_favorite { "★" } else { "☆" };
            info_row = info_row.push(Space::with_width(Length::Fill));
            info_row = info_row.push(text(fav_icon).size(12));
        }

        let card_content = column![thumbnail_display, title, info_row]
            .spacing(8)
            .padding(10)
            .width(Length::Fixed(self.config.width));

        button(card_content)
            .style(if self.is_selected {
                button::primary
            } else {
                button::secondary
            })
            .padding(0)
            .on_press(on_message(CardMessage::Selected(id)))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_config_default() {
        let config = CardConfig::default();
        assert_eq!(config.width, 200.0);
        assert!(config.show_type_badge);
    }
}
