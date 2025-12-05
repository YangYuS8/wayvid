//! Library view - wallpaper grid browser with detail panel
//!
//! Displays wallpapers in a responsive grid with thumbnails,
//! search, filtering, and a right-side detail panel.

use iced::widget::{
    button, column, container, horizontal_space, image, row, scrollable, text, text_input, Space,
};
use iced::{Element, Length};
use rust_i18n::t;

use crate::messages::Message;
use crate::state::{AppState, SourceFilter, ThumbnailState, WallpaperFilter};

/// Detail panel width (wider for better content display)
const DETAIL_PANEL_WIDTH: f32 = 320.0;

/// Safely truncate a string to a maximum number of characters (UTF-8 aware)
fn truncate_string(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        let truncated: String = chars[..max_chars.saturating_sub(3)].iter().collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

/// Render the library view with grid and detail panel
pub fn view(state: &AppState) -> Element<'_, Message> {
    // Header with search and filters
    let header = view_header(state);

    // Main content area: grid + detail panel
    let main_content = if state.detail_panel_visible {
        // Two-column layout: grid on left, detail panel on right
        let grid = view_grid(state);
        let detail = view_detail_panel(state);

        row![
            container(grid).width(Length::Fill),
            container(detail).width(Length::Fixed(DETAIL_PANEL_WIDTH))
        ]
        .spacing(10)
        .into()
    } else {
        // Single column: just the grid
        view_grid(state)
    };

    // Status bar
    let status = view_status(state);

    column![header, main_content, status]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Header with search bar and filter buttons
fn view_header(state: &AppState) -> Element<'_, Message> {
    // Search input
    let search = text_input(&t!("library.search_placeholder"), &state.search_query)
        .on_input(Message::SearchChanged)
        .padding(8)
        .width(Length::Fixed(250.0));

    // Source filter buttons (Workshop / Local / All)
    let source_buttons: Vec<Element<Message>> = SourceFilter::all()
        .iter()
        .map(|source| {
            let is_active = state.source_filter == *source;
            button(text(source.name()).size(13))
                .padding([4, 8])
                .style(if is_active {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Message::SourceFilterChanged(*source))
                .into()
        })
        .collect();

    let source_filters = row(source_buttons).spacing(4);

    // Type filter buttons
    let filter_buttons: Vec<Element<Message>> = WallpaperFilter::all()
        .iter()
        .map(|filter| {
            let is_active = state.current_filter == *filter;
            button(text(filter.name()).size(12))
                .padding([3, 6])
                .style(if is_active {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Message::FilterChanged(*filter))
                .into()
        })
        .collect();

    let type_filters = row(filter_buttons).spacing(4);

    // Detail panel toggle button
    let detail_toggle = button(text(if state.detail_panel_visible {
        "▶"
    } else {
        "◀"
    }))
    .padding([4, 8])
    .style(button::text)
    .on_press(Message::ToggleDetailPanel);

    column![
        row![search, horizontal_space(), source_filters, detail_toggle].spacing(15),
        type_filters,
    ]
    .spacing(8)
    .into()
}

/// Wallpaper grid with responsive columns
fn view_grid(state: &AppState) -> Element<'_, Message> {
    let wallpapers = state.filtered_wallpapers();

    if wallpapers.is_empty() {
        let empty_view = column![
            text("[Empty]").size(32),
            Space::with_height(10),
            text(t!("library.no_wallpapers").to_string()).size(20),
            Space::with_height(5),
            text(t!("library.add_folders_hint").to_string()).size(14),
            Space::with_height(15),
            button(text(t!("library.add_folder").to_string()))
                .padding([8, 16])
                .on_press(Message::AddFolder),
        ]
        .spacing(5)
        .align_x(iced::Alignment::Center);

        return container(empty_view)
            .padding(40)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    // Create grid of wallpaper cards with fixed spacing
    let card_spacing = 10.0; // Fixed spacing between cards

    let cards: Vec<Element<Message>> = wallpapers
        .into_iter()
        .map(|wp| wallpaper_card(wp, state))
        .collect();

    // Use row with wrap behavior - cards flow naturally based on available width
    let grid_content = row(cards).spacing(card_spacing).wrap();

    scrollable(container(grid_content).padding(10).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Single wallpaper card (compact version for grid)
fn wallpaper_card<'a>(
    wallpaper: &'a wayvid_core::WallpaperItem,
    state: &'a AppState,
) -> Element<'a, Message> {
    let id = wallpaper.id.clone();
    let is_selected = state.selected_wallpaper.as_ref() == Some(&id);

    // Get thumbnail state and display accordingly
    let thumbnail_state = state.get_thumbnail_state(&id);
    let thumb_size = 120.0; // Fixed 1:1 square thumbnail

    let thumbnail: Element<Message> = match thumbnail_state {
        ThumbnailState::Loaded => {
            if let Some(data) = state.thumbnails.get(&id) {
                let handle = image::Handle::from_bytes(data.clone());
                container(
                    image(handle)
                        .width(Length::Fixed(thumb_size))
                        .height(Length::Fixed(thumb_size))
                        .content_fit(iced::ContentFit::Cover),
                )
                .width(Length::Fixed(thumb_size))
                .height(Length::Fixed(thumb_size))
                .into()
            } else {
                placeholder_thumbnail(thumb_size)
            }
        }
        ThumbnailState::Loading => container(
            column![text("...").size(20), text(t!("status.loading")).size(9)]
                .align_x(iced::Alignment::Center)
                .spacing(3),
        )
        .width(Length::Fixed(thumb_size))
        .height(Length::Fixed(thumb_size))
        .center_x(Length::Fixed(thumb_size))
        .center_y(Length::Fixed(thumb_size))
        .style(container::bordered_box)
        .into(),
        ThumbnailState::Failed(_) => container(
            column![text("[!]").size(20), text("Error").size(10)]
                .align_x(iced::Alignment::Center)
                .spacing(2),
        )
        .width(Length::Fixed(thumb_size))
        .height(Length::Fixed(thumb_size))
        .center_x(Length::Fixed(thumb_size))
        .center_y(Length::Fixed(thumb_size))
        .style(container::bordered_box)
        .into(),
        ThumbnailState::NotLoaded => placeholder_thumbnail(thumb_size),
    };

    // Truncate long names (UTF-8 safe)
    let name = truncate_string(&wallpaper.name, 18);

    let title = text(name).size(11);

    // Card content: thumbnail + title only (type shown in detail panel)
    let card_content = column![thumbnail, title]
        .spacing(2)
        .width(Length::Fixed(thumb_size));

    let card_style = if is_selected {
        button::primary
    } else {
        button::text
    };

    button(card_content)
        .style(card_style)
        .padding(2)
        .on_press(Message::SelectWallpaper(id.clone()))
        .into()
}

/// Placeholder thumbnail with fixed size
fn placeholder_thumbnail(size: f32) -> Element<'static, Message> {
    container(text("[?]").size(24))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fixed(size))
        .center_y(Length::Fixed(size))
        .style(container::bordered_box)
        .into()
}

/// Detail panel showing selected wallpaper info
fn view_detail_panel(state: &AppState) -> Element<'_, Message> {
    let content: Element<'_, Message> = if let Some(wallpaper) = state.get_selected_wallpaper() {
        // Selected wallpaper details
        let id = wallpaper.id.clone();

        // Large preview image
        let preview: Element<Message> = {
            let thumbnail_state = state.get_thumbnail_state(&id);
            let preview_height = 160.0;

            match thumbnail_state {
                ThumbnailState::Loaded => {
                    if let Some(data) = state.thumbnails.get(&id) {
                        let handle = image::Handle::from_bytes(data.clone());
                        container(
                            image(handle)
                                .width(Length::Fill)
                                .height(Length::Fixed(preview_height))
                                .content_fit(iced::ContentFit::Contain),
                        )
                        .width(Length::Fill)
                        .height(Length::Fixed(preview_height))
                        .style(container::bordered_box)
                        .into()
                    } else {
                        placeholder_thumbnail(preview_height)
                    }
                }
                _ => placeholder_thumbnail(preview_height),
            }
        };

        // Title
        let title = text(&wallpaper.name).size(16);

        // Author
        let author = wallpaper.metadata.author.as_deref().unwrap_or("Unknown");
        let author_row = row![text(t!("detail.author")).size(12), text(author).size(12)].spacing(5);

        // Type badge
        let type_name = format!("{:?}", wallpaper.wallpaper_type);
        let type_row = row![text(t!("detail.type")).size(12), text(type_name).size(12)].spacing(5);

        // Source badge
        let source_name = match wallpaper.source_type {
            wayvid_core::SourceType::SteamWorkshop => "Steam Workshop",
            wayvid_core::SourceType::LocalFile | wayvid_core::SourceType::LocalDirectory => "Local",
        };
        let source_row = row![
            text(t!("detail.source")).size(12),
            text(source_name).size(12)
        ]
        .spacing(5);

        // Tags (if any)
        let tags_section: Element<Message> = if !wallpaper.metadata.tags.is_empty() {
            let tags_text = wallpaper.metadata.tags.join(", ");
            column![text(t!("detail.tags")).size(12), text(tags_text).size(11)]
                .spacing(3)
                .into()
        } else {
            Space::with_height(0).into()
        };

        // Description (if any)
        let desc_section: Element<Message> = if let Some(desc) = &wallpaper.metadata.description {
            if !desc.is_empty() {
                let short_desc = truncate_string(desc, 150);
                column![
                    text(t!("detail.description")).size(12),
                    text(short_desc).size(11)
                ]
                .spacing(3)
                .into()
            } else {
                Space::with_height(0).into()
            }
        } else {
            Space::with_height(0).into()
        };

        // Action buttons
        let apply_button = button(
            row![text(">").size(14), text(t!("detail.apply")).size(13)]
                .spacing(5)
                .align_y(iced::Alignment::Center),
        )
        .padding([8, 16])
        .width(Length::Fill)
        .style(button::success)
        .on_press(Message::ApplyWallpaper(id.clone()));

        // Monitor selector (if multiple monitors)
        let monitor_section: Element<Message> = if state.monitors.len() > 1 {
            let monitor_buttons: Vec<Element<Message>> = state
                .monitors
                .iter()
                .map(|m| {
                    button(text(&m.name).size(11))
                        .padding([4, 8])
                        .style(button::secondary)
                        .on_press(Message::ApplyToMonitor(m.name.clone()))
                        .into()
                })
                .collect();

            column![
                text(t!("detail.apply_to_monitor")).size(12),
                row(monitor_buttons).spacing(4)
            ]
            .spacing(5)
            .into()
        } else {
            Space::with_height(0).into()
        };

        column![
            preview,
            Space::with_height(10),
            title,
            Space::with_height(8),
            author_row,
            type_row,
            source_row,
            Space::with_height(5),
            tags_section,
            desc_section,
            Space::with_height(Length::Fill),
            apply_button,
            monitor_section,
        ]
        .spacing(4)
        .padding(12)
        .width(Length::Fill)
        .into()
    } else {
        // No wallpaper selected - show placeholder
        container(
            column![
                text("[Select]").size(24),
                Space::with_height(10),
                text(t!("detail.select_wallpaper")).size(14),
            ]
            .spacing(5)
            .align_x(iced::Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(20)
        .into()
    };

    container(content)
        .style(container::bordered_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Status bar
fn view_status(state: &AppState) -> Element<'_, Message> {
    let wallpaper_count = state.filtered_wallpapers().len();
    let total_count = state.wallpapers.len();

    let count_text = if wallpaper_count == total_count {
        t!("library.count", count = total_count).to_string()
    } else {
        t!(
            "library.count_filtered",
            shown = wallpaper_count,
            total = total_count
        )
        .to_string()
    };

    let status_text = if state.workshop_scanning {
        format!("{} (Workshop)", t!("status.loading"))
    } else if state.loading {
        t!("status.loading").to_string()
    } else {
        String::new()
    };

    let workshop_status = if state.workshop_available {
        text("[W] Workshop").size(11)
    } else {
        text("").size(11)
    };

    row![
        text(count_text).size(11),
        horizontal_space(),
        workshop_status,
        text(status_text).size(11),
    ]
    .spacing(8)
    .padding(4)
    .into()
}
