use crate::action_outcome::{ActionOutcome, InvalidatedPage};
use crate::assembly::settings_page::assemble_settings_page;
use crate::models::{SettingsPageSnapshot, SettingsUpdateInput};
use crate::services::settings_service::SettingsService;

#[tauri::command]
pub fn load_settings_page() -> Result<SettingsPageSnapshot, String> {
    SettingsService::load_page().map(assemble_settings_page)
}

#[tauri::command]
pub fn update_settings(
    input: SettingsUpdateInput,
) -> Result<ActionOutcome<SettingsPageSnapshot>, String> {
    let snapshot = assemble_settings_page(SettingsService::update_settings(input)?);

    Ok(ActionOutcome {
        ok: true,
        message: Some("Settings updated".to_string()),
        shell_patch: None,
        current_update: Some(snapshot),
        invalidations: vec![InvalidatedPage::Settings],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_snapshot_uses_real_settings_fields() {
        let snapshot = load_settings_page().unwrap();

        assert!(!snapshot.language.is_empty());
        assert!(!snapshot.theme.is_empty());
        assert!(snapshot.steam_status_message.contains("Steam"));
    }
}
