//! Library view - wallpaper grid browser
//!
//! Displays wallpapers in a responsive grid with thumbnails,
//! search, and filtering capabilities.

use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input, Space,
};
use iced::{Element, Length};

use crate::messages::Message;
use crate::state::{AppState, WallpaperFilter};

/// Render the library view
pub fn view(state: &AppState) -> Element<Message> {
    // Header with search and filters
    let header = view_header(state);

    // Wallpaper grid
    let grid = view_grid(state);

    // Status bar
    let status = view_status(state);

    column![header, grid, status]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Header with search bar and filter buttons
fn view_header(state: &AppState) -> Element<Message> {
    // Search input
    let search = text_input("Search wallpapers...", &state.search_query)
        .on_input(Message::SearchChanged)
        .padding(10)
        .width(Length::Fixed(300.0));

    // Filter buttons
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

    let filters = row(filter_buttons).spacing(5);

    row![search, horizontal_space(), filters]
        .spacing(20)
        .into()
}

/// Wallpaper grid
fn view_grid(state: &AppState) -> Element<Message> {
    let wallpapers = state.filtered_wallpapers();

    if wallpapers.is_empty() {
        let empty_view = column![
            text("No wallpapers found").size(24),
            Space::with_height(10),
            text("Add folders to your library to get started"),
            Space::with_height(20),
            button("Add Folder")
                .padding(10)
                .on_press(Message::AddFolder),
        ]
        .spacing(5)
        .align_x(iced::Alignment::Center);

        return container(empty_view)
            .center(Length::Fill)
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

    let grid_content = column(rows).spacing(15);

    scrollable(grid_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Single wallpaper card
fn wallpaper_card<'a>(wallpaper: &'a wayvid_core::WallpaperItem, state: &'a AppState) -> Element<'a, Message> {
    let id = wallpaper.id.clone();
    let is_selected = state.selected_wallpaper.as_ref() == Some(&id);

    // Thumbnail (placeholder for now)
    let thumbnail: Element<Message> = if state.thumbnails.contains_key(&id) {
        // TODO: Load actual image from data
        container(text("🖼️").size(48))
            .width(Length::Fill)
            .height(Length::Fixed(120.0))
            .center(Length::Fill)
            .style(container::bordered_box)
            .into()
    } else {
        container(text(wallpaper.wallpaper_type.icon()).size(48))
            .width(Length::Fill)
            .height(Length::Fixed(120.0))
            .center(Length::Fill)
            .style(container::bordered_box)
            .into()
    };

    // Card title
    let name = wallpaper.name.clone();
    let title = text(name).size(14).width(Length::Fill);

    // Type badge
    let type_badge = text(format!("{:?}", wallpaper.wallpaper_type)).size(10);

    let card_content = column![thumbnail, title, type_badge]
        .spacing(5)
        .padding(10)
        .width(Length::FillPortion(1));

    let _card_style = if is_selected {
        container::bordered_box
    } else {
        container::bordered_box
    };

    // Make the card clickable
    button(card_content)
        .style(button::text)
        .padding(0)
        .on_press(Message::SelectWallpaper(id.clone()))
        // TODO: Double-click for apply
        .into()
}

/// Status bar
fn view_status(state: &AppState) -> Element<Message> {
    let wallpaper_count = state.filtered_wallpapers().len();
    let total_count = state.wallpapers.len();

    let count_text = if wallpaper_count == total_count {
        format!("{} wallpapers", total_count)
    } else {
        format!("{} of {} wallpapers", wallpaper_count, total_count)
    };

    let loading_text = if state.loading {
        "Loading..."
    } else {
        ""
    };

    row![
        text(count_text).size(12),
        horizontal_space(),
        text(loading_text).size(12),
    ]
    .padding(5)
    .into()
}
