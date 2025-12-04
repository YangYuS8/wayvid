//! Settings view - application configuration
//!
//! Configure playback, power management, and general options.

use iced::widget::{button, checkbox, column, container, horizontal_space, row, slider, text, Space};
use iced::{Element, Length};

use crate::messages::Message;
use crate::state::AppState;

/// Render the settings view
pub fn view(state: &AppState) -> Element<Message> {
    let header = text("Settings").size(24);

    // General section
    let general_section = section(
        "General",
        column![
            setting_row(
                "Start with system",
                "Launch wayvid when you log in",
                checkbox("", state.settings.autostart)
                    .on_toggle(Message::ToggleAutostart),
            ),
            setting_row(
                "Minimize to tray",
                "Keep running in background when window is closed",
                checkbox("", state.settings.minimize_to_tray)
                    .on_toggle(Message::ToggleMinimizeToTray),
            ),
        ]
        .spacing(15),
    );

    // Playback section
    let playback_section = section(
        "Playback",
        column![
            setting_row(
                "Volume",
                "Default volume for video wallpapers",
                row![
                    slider(0.0..=100.0, state.settings.volume * 100.0, |v| {
                        // TODO: Add volume message
                        Message::ToggleAutostart(false) // Placeholder
                    })
                    .width(Length::Fixed(200.0)),
                    text(format!("{}%", (state.settings.volume * 100.0) as u32)),
                ]
                .spacing(10),
            ),
            setting_row(
                "FPS Limit",
                "Maximum frame rate (leave empty for unlimited)",
                text(match state.settings.fps_limit {
                    Some(fps) => format!("{} FPS", fps),
                    None => "Unlimited".to_string(),
                }),
            ),
        ]
        .spacing(15),
    );

    // Power section
    let power_section = section(
        "Power Management",
        column![
            setting_row(
                "Pause on battery",
                "Pause wallpaper playback when on battery power",
                checkbox("", state.settings.pause_on_battery)
                    .on_toggle(|_| Message::ToggleAutostart(false)), // TODO: Add proper message
            ),
            setting_row(
                "Pause on fullscreen",
                "Pause when a fullscreen application is running",
                checkbox("", state.settings.pause_on_fullscreen)
                    .on_toggle(|_| Message::ToggleAutostart(false)), // TODO: Add proper message
            ),
        ]
        .spacing(15),
    );

    // Theme section
    let theme_section = section(
        "Appearance",
        column![
            setting_row(
                "Theme",
                "Switch between light and dark mode",
                button("Toggle Theme")
                    .padding([5, 10])
                    .on_press(Message::ToggleTheme),
            ),
        ]
        .spacing(15),
    );

    // Save button
    let actions = row![
        horizontal_space(),
        button("Save Settings")
            .padding([10, 30])
            .style(button::primary)
            .on_press(Message::SaveSettings),
    ];

    column![
        header,
        Space::with_height(20),
        general_section,
        playback_section,
        power_section,
        theme_section,
        Space::with_height(Length::Fill),
        actions,
    ]
    .spacing(20)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Section container
fn section<'a>(title: &'a str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    let section_content = column![
        text(title).size(18),
        Space::with_height(10),
        content.into(),
    ]
    .spacing(5)
    .padding(15);

    container(section_content)
        .style(container::bordered_box)
        .width(Length::Fill)
        .into()
}

/// Single setting row
fn setting_row<'a>(
    label: &'a str,
    description: &'a str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let label_col = column![
        text(label).size(14),
        text(description).size(12),
    ]
    .spacing(2);

    row![label_col, horizontal_space(), control.into()]
        .align_y(iced::Alignment::Center)
        .into()
}
