//! Library view - wallpaper grid browser
//!
//! Displays wallpapers in a responsive grid with thumbnails,
//! search, and filtering capabilities.

use iced::widget::{
    button, column, container, horizontal_space, image, row, scrollable, text, text_input, Space,
};
use iced::{Element, Length};
use rust_i18n::t;

use crate::messages::Message;
use crate::state::{AppState, SourceFilter, ThumbnailState, WallpaperFilter};

/// Render the library view
pub fn view(state: &AppState) -> Element<'_, Message> {
    // Header with search and filters
    let header = view_header(state);

    // Wallpaper grid
    let grid = view_grid(state);

    // Status bar
    let status = view_status(state);

    column![header, grid, status]
        .spacing(10)
        .width(Length::Fill)
        .into()
}

/// Header with search bar and filter buttons
fn view_header(state: &AppState) -> Element<'_, Message> {
    // Search input
    let search = text_input(
        &t!("library.search_placeholder").to_string(),
        &state.search_query,
    )
    .on_input(Message::SearchChanged)
    .padding(10)
    .width(Length::Fixed(300.0));

    // Source filter buttons (Workshop / Local / All)
    let source_buttons: Vec<Element<Message>> = SourceFilter::all()
        .iter()
        .map(|source| {
            let is_active = state.source_filter == *source;
            button(text(source.name()))
                .padding(5)
                .style(if is_active {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Message::SourceFilterChanged(*source))
                .into()
        })
        .collect();

    let source_filters = row(source_buttons).spacing(5);

    // Type filter buttons
    let filter_buttons: Vec<Element<Message>> = WallpaperFilter::all()
        .iter()
        .map(|filter| {
            let is_active = state.current_filter == *filter;
            button(text(filter.name()))
                .padding(5)
                .style(if is_active {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Message::FilterChanged(*filter))
                .into()
        })
        .collect();

    let type_filters = row(filter_buttons).spacing(5);

    column![
        row![search, horizontal_space(), source_filters].spacing(20),
        type_filters,
    ]
    .spacing(10)
    .into()
}

/// Wallpaper grid
fn view_grid(state: &AppState) -> Element<'_, Message> {
    let wallpapers = state.filtered_wallpapers();

    if wallpapers.is_empty() {
        let empty_view = column![
            text(t!("library.no_wallpapers").to_string()).size(24),
            Space::with_height(10),
            text(t!("library.add_folders_hint").to_string()),
            Space::with_height(20),
            button(text(t!("library.add_folder").to_string()))
                .padding(10)
                .on_press(Message::AddFolder),
        ]
        .spacing(5)
        .align_x(iced::Alignment::Center);

        return container(empty_view)
            .padding(40)
            .center_x(Length::Fill)
            .into();
    }

    // Create grid of wallpaper cards
    let cards: Vec<Element<Message>> = wallpapers
        .into_iter()
        .map(|wp| wallpaper_card(wp, state))
        .collect();

    // Wrap cards in rows (4 per row for now)
    let mut rows: Vec<Element<Message>> = Vec::new();
    let mut current_row: Vec<Element<Message>> = Vec::new();
    let cards_per_row = 4;

    for card in cards {
        current_row.push(card);
        if current_row.len() >= cards_per_row {
            rows.push(row(std::mem::take(&mut current_row)).spacing(15).into());
        }
    }

    // Don't forget the last partial row
    if !current_row.is_empty() {
        // Pad with empty space to maintain alignment
        while current_row.len() < cards_per_row {
            current_row.push(Space::with_width(Length::FillPortion(1)).into());
        }
        rows.push(row(current_row).spacing(15).into());
    }

    let grid_content = container(column(rows).spacing(15))
        .width(Length::Fill)
        .padding(10);

    // Use scrollable with explicit shrink for content
    scrollable(grid_content).width(Length::Fill).into()
}

/// Single wallpaper card
fn wallpaper_card<'a>(
    wallpaper: &'a wayvid_core::WallpaperItem,
    state: &'a AppState,
) -> Element<'a, Message> {
    let id = wallpaper.id.clone();
    let is_selected = state.selected_wallpaper.as_ref() == Some(&id);

    // Get thumbnail state and display accordingly
    let thumbnail_state = state.get_thumbnail_state(&id);

    let thumbnail: Element<Message> = match thumbnail_state {
        ThumbnailState::Loaded => {
            // Display actual thumbnail image from cached data
            if let Some(data) = state.thumbnails.get(&id) {
                let handle = image::Handle::from_bytes(data.clone());
                container(
                    image(handle)
                        .width(Length::Fill)
                        .height(Length::Fixed(120.0))
                        .content_fit(iced::ContentFit::Cover),
                )
                .width(Length::Fill)
                .height(Length::Fixed(120.0))
                .style(container::bordered_box)
                .into()
            } else {
                // Fallback if data is missing (shouldn't happen)
                container(text(wallpaper.wallpaper_type.icon()).size(48))
                    .width(Length::Fill)
                    .height(Length::Fixed(120.0))
                    .center_x(Length::Fill)
                    .center_y(Length::Fixed(120.0))
                    .style(container::bordered_box)
                    .into()
            }
        }
        ThumbnailState::Loading => {
            // Show loading indicator
            container(
                column![text("⏳").size(32), text(t!("status.loading")).size(10)]
                    .align_x(iced::Alignment::Center)
                    .spacing(5),
            )
            .width(Length::Fill)
            .height(Length::Fixed(120.0))
            .center_x(Length::Fill)
            .center_y(Length::Fixed(120.0))
            .style(container::bordered_box)
            .into()
        }
        ThumbnailState::Failed(_) => {
            // Show error indicator with type icon fallback
            container(
                column![
                    text(wallpaper.wallpaper_type.icon()).size(32),
                    text("⚠").size(12)
                ]
                .align_x(iced::Alignment::Center)
                .spacing(3),
            )
            .width(Length::Fill)
            .height(Length::Fixed(120.0))
            .center_x(Length::Fill)
            .center_y(Length::Fixed(120.0))
            .style(container::bordered_box)
            .into()
        }
        ThumbnailState::NotLoaded => {
            // Show placeholder with type icon (will be loaded by subscription)
            let icon = if wallpaper.thumbnail_path.is_some() {
                "📷" // Has preview image, waiting to load
            } else {
                wallpaper.wallpaper_type.icon()
            };
            container(text(icon).size(48))
                .width(Length::Fill)
                .height(Length::Fixed(120.0))
                .center_x(Length::Fill)
                .center_y(Length::Fixed(120.0))
                .style(container::bordered_box)
                .into()
        }
    };

    // Card title
    let name = wallpaper.name.clone();
    let title = text(name).size(14).width(Length::Fill);

    // Source and type badge
    let source_icon = match wallpaper.source_type {
        wayvid_core::SourceType::SteamWorkshop => "🎮",
        wayvid_core::SourceType::LocalFile | wayvid_core::SourceType::LocalDirectory => "📁",
    };
    let type_badge = text(format!("{} {:?}", source_icon, wallpaper.wallpaper_type)).size(10);

    let card_content = column![thumbnail, title, type_badge]
        .spacing(5)
        .padding(10)
        .width(Length::FillPortion(1));

    // Card style based on selection
    let card_style = if is_selected {
        button::primary
    } else {
        button::text
    };

    // Make the card clickable
    button(card_content)
        .style(card_style)
        .padding(0)
        .on_press(Message::SelectWallpaper(id.clone()))
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
        t!("status.loading").to_string() + " (Workshop)"
    } else if state.loading {
        t!("status.loading").to_string()
    } else {
        String::new()
    };

    // Workshop availability indicator
    let workshop_status = if state.workshop_available {
        text("🎮 Workshop").size(12)
    } else {
        text("").size(12)
    };

    row![
        text(count_text).size(12),
        horizontal_space(),
        workshop_status,
        text(status_text).size(12),
    ]
    .spacing(10)
    .padding(5)
    .into()
}
