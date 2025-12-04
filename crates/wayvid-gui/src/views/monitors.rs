//! Monitor management view for wayvid-gui

use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, vertical_space,
};
use iced::{Element, Length};
use rust_i18n::t;

use crate::state::{MonitorInfo, WayvidState};
use crate::widgets::MonitorSelector;

/// Monitor view state
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Fields reserved for preview feature
pub struct MonitorView {
    selected_monitor: Option<String>,
    show_preview: bool,
}

impl MonitorView {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)] // Reserved for monitor details panel
    pub fn selected_monitor(&self) -> Option<&str> {
        self.selected_monitor.as_deref()
    }

    pub fn select_monitor(&mut self, name: String) {
        self.selected_monitor = Some(name);
    }

    #[allow(dead_code)] // Reserved for monitor details panel
    pub fn clear_selection(&mut self) {
        self.selected_monitor = None;
    }

    #[allow(dead_code)] // Reserved for preview toggle
    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }
}

/// Render the monitors management view
pub fn view<'a, M: 'a + Clone>(
    state: &'a WayvidState,
    monitor_view: &'a MonitorView,
    on_select: impl Fn(String) -> M + 'a,
    on_apply: impl Fn(String) -> M + 'a,
    on_clear: impl Fn(String) -> M + 'a,
    on_refresh: M,
) -> Element<'a, M> {
    let title = text(t!("monitors.title").to_string()).size(24);

    let monitors = &state.monitors;

    // Monitor selector widget
    let selector = MonitorSelector::new(monitors)
        .selected(monitor_view.selected_monitor.as_deref())
        .on_select(on_select);

    // Monitor details panel
    let details_panel = if let Some(selected_name) = &monitor_view.selected_monitor {
        if let Some(monitor) = monitors.iter().find(|m| &m.name == selected_name) {
            monitor_details_panel(monitor, on_apply, on_clear)
        } else {
            container(text(t!("monitors.no_monitors").to_string()))
                .padding(20)
                .into()
        }
    } else {
        container(text(t!("monitors.select_wallpaper").to_string()))
            .padding(20)
            .into()
    };

    // Monitor list (for accessibility / alternative view)
    let monitor_list = monitor_list_view(monitors, monitor_view.selected_monitor.as_deref());

    let refresh_btn = button(text(t!("monitors.refresh").to_string()))
        .padding([8, 16])
        .on_press(on_refresh);

    let content = column![
        row![title, horizontal_space(), refresh_btn,].spacing(10),
        vertical_space().height(20),
        row![
            container(selector.view()).width(Length::FillPortion(2)),
            container(details_panel)
                .width(Length::FillPortion(1))
                .padding(10),
        ]
        .spacing(20),
        vertical_space().height(20),
        text(t!("nav.monitors").to_string()).size(18),
        vertical_space().height(10),
        scrollable(monitor_list),
    ]
    .spacing(10)
    .padding(20);

    container(content).width(Length::Fill).into()
}

fn monitor_details_panel<'a, M: 'a + Clone>(
    monitor: &'a MonitorInfo,
    on_apply: impl Fn(String) -> M + 'a,
    on_clear: impl Fn(String) -> M + 'a,
) -> Element<'a, M> {
    let name = monitor.name.clone();
    let name2 = monitor.name.clone();

    let details = column![
        text(&monitor.name).size(18),
        vertical_space().height(10),
        text(
            t!(
                "monitors.resolution",
                width = monitor.width,
                height = monitor.height
            )
            .to_string()
        )
        .size(14),
        text(format!("Position: ({}, {})", monitor.x, monitor.y)).size(14),
        text(format!("Scale: {:.1}x", monitor.scale)).size(14),
        if monitor.primary {
            text(t!("monitors.primary").to_string()).size(14)
        } else {
            text("").size(14)
        },
        vertical_space().height(20),
        if let Some(ref wallpaper) = monitor.current_wallpaper {
            let wp_name = wallpaper
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| wallpaper.display().to_string());
            text(t!("monitors.current_wallpaper", name = wp_name).to_string()).size(12)
        } else {
            text(t!("monitors.no_wallpaper").to_string()).size(12)
        },
        vertical_space().height(20),
        row![
            button(text(t!("monitors.apply").to_string()))
                .padding([8, 16])
                .on_press(on_apply(name)),
            button(text(t!("monitors.clear").to_string()))
                .padding([8, 16])
                .on_press(on_clear(name2)),
        ]
        .spacing(10),
    ]
    .spacing(5);

    container(details).padding(20).into()
}

fn monitor_list_view<'a, M: 'a>(
    monitors: &'a [MonitorInfo],
    selected: Option<&str>,
) -> Element<'a, M> {
    let items: Vec<Element<'a, M>> = monitors
        .iter()
        .map(|m| {
            let is_selected = selected.is_some_and(|s| s == m.name);

            let style_text = if is_selected { "► " } else { "" };

            let primary_text = if m.primary {
                format!(" ({})", t!("monitors.primary"))
            } else {
                String::new()
            };

            let item = row![
                text(format!("{}{}", style_text, m.name)).size(14),
                horizontal_space(),
                text(format!("{}x{}", m.width, m.height)).size(12),
                text(primary_text).size(12),
            ]
            .spacing(10)
            .padding(8);

            container(item).width(Length::Fill).into()
        })
        .collect();

    column(items).spacing(4).width(Length::Fill).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_view_selection() {
        let mut view = MonitorView::new();

        assert!(view.selected_monitor().is_none());

        view.select_monitor("eDP-1".to_string());
        assert_eq!(view.selected_monitor(), Some("eDP-1"));

        view.clear_selection();
        assert!(view.selected_monitor().is_none());
    }

    #[test]
    fn test_monitor_view_preview_toggle() {
        let mut view = MonitorView::new();

        assert!(!view.show_preview);

        view.toggle_preview();
        assert!(view.show_preview);

        view.toggle_preview();
        assert!(!view.show_preview);
    }
}
