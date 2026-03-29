use crate::models::SettingsPageSnapshot;

#[tauri::command]
pub fn load_settings_page() -> Result<SettingsPageSnapshot, String> {
    crate::settings::load_settings_page()
}
