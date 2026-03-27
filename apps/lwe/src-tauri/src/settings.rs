use crate::models::SettingsPageSnapshot;

#[tauri::command]
pub fn load_settings_page() -> SettingsPageSnapshot {
    SettingsPageSnapshot {
        language: "en".to_string(),
        theme: "system".to_string(),
        steam_required: true,
        stale: false,
    }
}
