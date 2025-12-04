//! About view - application information
//!
//! Shows version, credits, and links.

use iced::widget::{button, column, container, horizontal_rule, row, text, Space};
use iced::{Element, Length};

use crate::messages::Message;
use crate::state::AppState;

/// Render the about view
pub fn view(_state: &AppState) -> Element<Message> {
    let logo = text("🎬 Wayvid").size(48);
    
    let version = text(format!("Version {}", env!("CARGO_PKG_VERSION"))).size(16);

    let description = text(
        "A lightweight, high-performance dynamic video wallpaper application for Wayland."
    ).size(14);

    let features = column![
        text("Features:").size(16),
        Space::with_height(5),
        text("• Native Wayland layer-shell support"),
        text("• Hardware-accelerated video decoding"),
        text("• HDR and 10-bit video support"),
        text("• Steam Workshop compatibility"),
        text("• Multi-monitor with per-output configuration"),
        text("• Power-aware playback"),
    ]
    .spacing(3);

    let links = row![
        link_button("GitHub", "https://github.com/YangYuS8/wayvid"),
        link_button("Documentation", "https://docs.wayvid.dev"),
        link_button("Report Issue", "https://github.com/YangYuS8/wayvid/issues"),
    ]
    .spacing(10);

    let credits = column![
        text("Credits").size(16),
        Space::with_height(5),
        text("Built with:"),
        text("• Rust programming language"),
        text("• iced GUI framework"),
        text("• libmpv for video playback"),
        text("• smithay-client-toolkit for Wayland"),
    ]
    .spacing(3);

    let license = text("Licensed under MIT or Apache-2.0").size(12);

    let content = column![
        logo,
        version,
        Space::with_height(10),
        description,
        Space::with_height(20),
        horizontal_rule(1),
        Space::with_height(20),
        features,
        Space::with_height(20),
        links,
        Space::with_height(20),
        horizontal_rule(1),
        Space::with_height(20),
        credits,
        Space::with_height(Length::Fill),
        license,
    ]
    .spacing(5)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);

    container(content)
        .center_x(Length::Fill)
        .padding(20)
        .into()
}

/// Create a link button
fn link_button<'a>(label: &'a str, _url: &str) -> Element<'a, Message> {
    // TODO: Add actual link opening functionality
    button(text(label).size(14))
        .padding([5, 15])
        .style(button::secondary)
        .into()
}
