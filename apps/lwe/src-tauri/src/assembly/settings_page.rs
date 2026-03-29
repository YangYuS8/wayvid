use crate::models::SettingsPageSnapshot;
use crate::results::settings::SettingsPageResult;

pub fn assemble_settings_page(result: SettingsPageResult) -> SettingsPageSnapshot {
    SettingsPageSnapshot {
        language: result.language,
        theme: result.theme,
        steam_required: result.steam_required,
        stale: result.stale,
    }
}
