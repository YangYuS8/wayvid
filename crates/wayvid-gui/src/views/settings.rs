//! Settings view - application configuration
//!
//! Configure playback, power management, and general options.

use iced::widget::{
    button, checkbox, column, container, horizontal_space, pick_list, row, slider, text, Space,
};
use iced::{Element, Length};
use rust_i18n::t;

use crate::i18n::Language;
use crate::messages::Message;
use crate::state::AppState;

/// Render the settings view
pub fn view(state: &AppState) -> Element<'_, Message> {
    let header = text(t!("settings.title").to_string()).size(24);

    // General section
    let general_section = section(
        &t!("settings.general"),
        column![
            setting_row(
                &t!("settings.autostart"),
                &t!("settings.autostart_desc"),
                checkbox("", state.app_settings.autostart.enabled).on_toggle(Message::ToggleAutostart),
            ),
            setting_row(
                &t!("settings.minimize_to_tray"),
                &t!("settings.minimize_to_tray_desc"),
                checkbox("", state.app_settings.gui.minimize_to_tray)
                    .on_toggle(Message::ToggleMinimizeToTray),
            ),
        ]
        .spacing(15),
    );

    // Playback section
    let volume_pct = (state.app_settings.playback.volume * 100.0) as u32;
    let playback_section = section(
        &t!("settings.playback"),
        column![
            setting_row(
                &t!("settings.volume"),
                &t!("settings.volume_desc"),
                row![
                    slider(0.0..=100.0, state.app_settings.playback.volume * 100.0, |v| {
                        Message::VolumeChanged(v / 100.0)
                    })
                    .width(Length::Fixed(200.0)),
                    text(format!("{}%", volume_pct)),
                ]
                .spacing(10),
            ),
            setting_row(
                &t!("settings.fps_limit"),
                &t!("settings.fps_limit_desc"),
                text(match state.app_settings.playback.fps_limit {
                    Some(fps) => format!("{} FPS", fps),
                    None => t!("settings.unlimited").to_string(),
                }),
            ),
        ]
        .spacing(15),
    );

    // Power section
    let power_section = section(
        &t!("settings.power"),
        column![
            setting_row(
                &t!("settings.pause_on_battery"),
                &t!("settings.pause_on_battery_desc"),
                checkbox("", state.app_settings.power.pause_on_battery)
                    .on_toggle(Message::TogglePauseOnBattery),
            ),
            setting_row(
                &t!("settings.pause_on_fullscreen"),
                &t!("settings.pause_on_fullscreen_desc"),
                checkbox("", state.app_settings.power.pause_on_fullscreen)
                    .on_toggle(Message::TogglePauseOnFullscreen),
            ),
        ]
        .spacing(15),
    );

    // Appearance section
    let theme_section = section(
        &t!("settings.appearance"),
        column![setting_row(
            &t!("settings.theme"),
            &t!("settings.theme_desc"),
            button(text(t!("settings.toggle_theme").to_string()))
                .padding([5, 10])
                .on_press(Message::ToggleTheme),
        ),]
        .spacing(15),
    );

    // Language section
    let language_options: Vec<Language> = Language::all().to_vec();
    let current_language = Language::from_code(&state.app_settings.gui.language);
    let language_section = section(
        &t!("settings.language"),
        column![setting_row(
            &t!("settings.language"),
            &t!("settings.language_desc"),
            pick_list(
                language_options,
                Some(current_language),
                Message::LanguageChanged
            )
            .width(Length::Fixed(200.0)),
        ),]
        .spacing(15),
    );

    // Save button
    let actions = row![
        horizontal_space(),
        button(text(t!("settings.save").to_string()))
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
        language_section,
        Space::with_height(40),
        actions,
    ]
    .spacing(20)
    .width(Length::Fill)
    .into()
}

/// Section container
fn section<'a>(title: &str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    let section_content = column![
        text(title.to_string()).size(18),
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
    label: &str,
    description: &str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let label_col = column![
        text(label.to_string()).size(14),
        text(description.to_string()).size(12),
    ]
    .spacing(2);

    row![label_col, horizontal_space(), control.into()]
        .align_y(iced::Alignment::Center)
        .into()
}
