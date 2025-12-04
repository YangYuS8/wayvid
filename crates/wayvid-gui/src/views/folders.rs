//! Folders view - manage wallpaper source folders
//!
//! Add, remove, and configure folders that contain wallpapers.

use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::messages::Message;
use crate::state::AppState;

/// Render the folders view
pub fn view(state: &AppState) -> Element<Message> {
    let header = row![
        text("Wallpaper Folders").size(24),
        horizontal_space(),
        button("Add Folder")
            .padding(10)
            .on_press(Message::AddFolder),
    ]
    .padding(10);

    let content: Element<Message> = if state.folders.is_empty() {
        let empty_view = column![
            text("No folders configured").size(18),
            Space::with_height(10),
            text("Add folders containing your wallpapers to build your library."),
            Space::with_height(20),
            button("Add Your First Folder")
                .padding(10)
                .on_press(Message::AddFolder),
        ]
        .spacing(5)
        .align_x(iced::Alignment::Center);

        container(empty_view)
            .center(Length::Fill)
            .into()
    } else {
        let folder_list: Vec<Element<Message>> = state
            .folders
            .iter()
            .map(|folder| folder_row(folder))
            .collect();

        scrollable(column(folder_list).spacing(10))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    column![header, content]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Single folder row
fn folder_row(folder: &crate::state::FolderEntry) -> Element<Message> {
    let path_str = folder.path.display().to_string();
    let wallpaper_count = folder.wallpaper_count;
    let last_scan = folder.last_scan.clone().unwrap_or_else(|| "Never scanned".to_string());
    let folder_path = folder.path.clone();
    let folder_path2 = folder.path.clone();
    
    let info = column![
        text(path_str).size(14),
        row![
            text(format!("{} wallpapers", wallpaper_count)).size(12),
            text(" • ").size(12),
            text(last_scan).size(12),
        ],
    ]
    .spacing(2);

    let actions = row![
        button("Scan")
            .padding(5)
            .on_press(Message::ScanFolder(folder_path)),
        button("Remove")
            .padding(5)
            .style(button::danger)
            .on_press(Message::RemoveFolder(folder_path2)),
    ]
    .spacing(5);

    let content = row![info, horizontal_space(), actions]
        .padding(15)
        .align_y(iced::Alignment::Center);

    container(content)
        .style(container::bordered_box)
        .width(Length::Fill)
        .into()
}
