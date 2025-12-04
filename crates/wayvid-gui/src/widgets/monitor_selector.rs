//! Monitor selector widget for multi-display setup

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::state::MonitorInfo;

/// Monitor selector configuration
#[derive(Debug, Clone)]
pub struct MonitorSelectorConfig {
    pub preview_scale: f32,
    pub min_preview_width: f32,
    pub max_preview_width: f32,
    pub show_names: bool,
    pub show_resolution: bool,
}

impl Default for MonitorSelectorConfig {
    fn default() -> Self {
        Self {
            preview_scale: 0.1,
            min_preview_width: 100.0,
            max_preview_width: 300.0,
            show_names: true,
            show_resolution: true,
        }
    }
}

/// Monitor selector widget
pub struct MonitorSelector<'a, M> {
    monitors: &'a [MonitorInfo],
    selected: Option<&'a str>,
    config: MonitorSelectorConfig,
    on_select: Option<Box<dyn Fn(String) -> M + 'a>>,
}

impl<'a, M: Clone + 'a> MonitorSelector<'a, M> {
    pub fn new(monitors: &'a [MonitorInfo]) -> Self {
        Self {
            monitors,
            selected: None,
            config: MonitorSelectorConfig::default(),
            on_select: None,
        }
    }

    pub fn selected(mut self, name: Option<&'a str>) -> Self {
        self.selected = name;
        self
    }

    pub fn on_select(mut self, f: impl Fn(String) -> M + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn view(self) -> Element<'a, M> {
        if self.monitors.is_empty() {
            return container(
                column![
                    text("No monitors detected").size(16),
                    text("Make sure your display server is running"),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .center(Length::Fill)
            .into();
        }

        let monitor_cards: Vec<Element<'a, M>> = self
            .monitors
            .iter()
            .map(|monitor| {
                let is_selected = self.selected.is_some_and(|s| s == monitor.name);
                self.build_monitor_card(monitor, is_selected)
            })
            .collect();

        let monitors_row = row(monitor_cards).spacing(20).align_y(Alignment::End);

        column![text("Select Monitor").size(18), Space::with_height(10), monitors_row]
            .spacing(10)
            .into()
    }

    fn build_monitor_card(
        &self,
        monitor: &'a MonitorInfo,
        is_selected: bool,
    ) -> Element<'a, M> {
        let scaled_width = (monitor.width as f32 * self.config.preview_scale)
            .max(self.config.min_preview_width)
            .min(self.config.max_preview_width);
        let aspect = monitor.height as f32 / monitor.width as f32;
        let scaled_height = scaled_width * aspect;

        let preview_content = column![
            text(&monitor.name).size(14),
            if self.config.show_resolution {
                text(format!("{}x{}", monitor.width, monitor.height)).size(10)
            } else {
                text("")
            },
        ]
        .align_x(Alignment::Center)
        .spacing(4);

        let preview = container(preview_content)
            .width(Length::Fixed(scaled_width))
            .height(Length::Fixed(scaled_height))
            .center(Length::Fill)
            .style(container::bordered_box);

        let is_primary = monitor.primary;
        let status = row![
            text(if is_primary { "●" } else { "○" }).style(move |_| text::Style {
                color: Some(if is_primary {
                    iced::Color::from_rgb(0.0, 0.8, 0.0)
                } else {
                    iced::Color::from_rgb(0.5, 0.5, 0.5)
                }),
            }),
            text(if monitor.primary { "Primary" } else { "" }).size(10),
        ]
        .spacing(4);

        let wallpaper_info = if let Some(ref wp) = monitor.current_wallpaper {
            text(wp.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Wallpaper".into())).size(10)
        } else {
            text("No wallpaper").size(10)
        };

        let card_content = column![preview, status, wallpaper_info]
            .spacing(6)
            .align_x(Alignment::Center);

        let mut btn = button(card_content)
            .style(if is_selected { button::primary } else { button::secondary })
            .padding(8);

        if let Some(ref on_select) = self.on_select {
            btn = btn.on_press(on_select(monitor.name.clone()));
        }

        btn.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = MonitorSelectorConfig::default();
        assert_eq!(config.preview_scale, 0.1);
        assert!(config.show_names);
    }

    #[test]
    fn test_preview_position() {
        let monitors = vec![MonitorInfo {
            name: "Test".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            scale: 1.0,
            primary: true,
            current_wallpaper: None,
        }];

        let selector: MonitorSelector<'_, ()> = MonitorSelector::new(&monitors);
        let config = &selector.config;
        let scaled_width = (1920.0 * config.preview_scale)
            .max(config.min_preview_width)
            .min(config.max_preview_width);
        assert!(scaled_width >= 100.0);
    }
}
