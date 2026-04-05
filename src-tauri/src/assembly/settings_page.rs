use crate::models::SettingsPageSnapshot;
use crate::services::settings_service::SettingsPageData;

pub(crate) fn assemble_settings_page(result: SettingsPageData) -> SettingsPageSnapshot {
    SettingsPageSnapshot {
        language: result.language,
        theme: result.theme,
        launch_on_login: result.launch_on_login,
        launch_on_login_available: result.launch_on_login_available,
        steam_web_api_key: result.steam_web_api_key,
        workshop_query: result.workshop_query,
        workshop_age_ratings: result.workshop_age_ratings,
        workshop_item_types: result.workshop_item_types,
        steam_required: result.steam_required,
        steam_status_message: result.steam_status_message,
        stale: result.stale,
    }
}
