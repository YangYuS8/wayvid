use crate::assembly::settings_page::assemble_settings_page;
use crate::models::SettingsPageSnapshot;
use crate::services::settings_service::SettingsService;

#[tauri::command]
pub fn load_settings_page() -> Result<SettingsPageSnapshot, String> {
    SettingsService::load_page().map(assemble_settings_page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_settings_snapshot_preserves_current_values() {
        let snapshot = load_settings_page().unwrap();

        assert_eq!(snapshot.language, "en");
        assert_eq!(snapshot.theme, "system");
        assert!(snapshot.steam_required);
        assert!(!snapshot.stale);
    }
}
