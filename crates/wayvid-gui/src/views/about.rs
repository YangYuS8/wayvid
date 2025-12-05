//! About view - application information
//!
//! Shows version, credits, and links.

use iced::widget::{button, column, container, horizontal_rule, row, svg, text, Space};
use iced::{Element, Length};
use rust_i18n::t;

use crate::messages::Message;
use crate::state::AppState;

/// Logo SVG data (embedded at compile time)
const LOGO_SVG: &[u8] = include_bytes!("../../../../logo.svg");

/// Render the about view
pub fn view(_state: &AppState) -> Element<'_, Message> {
    // Logo image using SVG widget
    let logo_handle = svg::Handle::from_memory(LOGO_SVG);
    let logo_image = svg(logo_handle)
        .width(Length::Fixed(64.0))
        .height(Length::Fixed(64.0));

    let logo_row = row![logo_image, text(t!("about.title").to_string()).size(48)]
        .spacing(16)
        .align_y(iced::Alignment::Center);

    let version =
        text(t!("about.version", version = env!("CARGO_PKG_VERSION")).to_string()).size(16);

    let description = text(t!("about.description").to_string()).size(14);

    let features = column![
        text(t!("about.features").to_string()).size(16),
        Space::with_height(5),
        text(format!("• {}", t!("about.feature_wayland"))),
        text(format!("• {}", t!("about.feature_hwdec"))),
        text(format!("• {}", t!("about.feature_hdr"))),
        text(format!("• {}", t!("about.feature_workshop"))),
        text(format!("• {}", t!("about.feature_multimon"))),
        text(format!("• {}", t!("about.feature_power"))),
    ]
    .spacing(3);

    // 先转换为 String 避免生命周期问题
    let github_label = t!("about.github").to_string();
    let docs_label = t!("about.docs").to_string();
    let issues_label = t!("about.issues").to_string();

    let links = row![
        link_button(github_label),
        link_button(docs_label),
        link_button(issues_label),
    ]
    .spacing(10);

    let credits = column![
        text(t!("about.credits").to_string()).size(16),
        Space::with_height(5),
        text(t!("about.built_with").to_string()),
        text("• Rust programming language"),
        text("• iced GUI framework"),
        text("• libmpv for video playback"),
        text("• smithay-client-toolkit for Wayland"),
    ]
    .spacing(3);

    let license = text(t!("about.license").to_string()).size(12);

    let content = column![
        logo_row,
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
        Space::with_height(40),
        license,
    ]
    .spacing(5)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);

    container(content)
        .center_x(Length::Fill)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Create a link button
fn link_button(label: String) -> Element<'static, Message> {
    // TODO: Add actual link opening functionality
    button(text(label).size(14))
        .padding([5, 15])
        .style(button::secondary)
        .into()
}
