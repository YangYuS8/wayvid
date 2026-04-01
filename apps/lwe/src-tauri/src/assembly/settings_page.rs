use crate::models::SettingsPageSnapshot;
use crate::results::settings::SettingsPageResult;

pub fn assemble_settings_page(result: SettingsPageResult) -> SettingsPageSnapshot {
    SettingsPageSnapshot {
        language: result.language,
        theme: result.theme,
        launch_on_login: false,
        launch_on_login_available: false,
        steam_required: result.steam_required,
        steam_status_message: if result.steam_required {
            "Steam is required for Workshop features".to_string()
        } else {
            "Steam is not required for current settings".to_string()
        },
        stale: result.stale,
    }
}
