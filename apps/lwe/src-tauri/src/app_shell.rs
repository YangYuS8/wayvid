use crate::library::load_library_projection;
use crate::models::AppShellSnapshot;
use crate::workshop::scan_workshop_catalog;

fn shell_snapshot() -> AppShellSnapshot {
    let workshop_items = scan_workshop_catalog();
    let library_items = load_library_projection();

    AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: crate::APP_CODE_NAME.to_string(),
        steam_available: wayvid_library::SteamLibrary::try_discover().is_some(),
        library_count: library_items.len(),
        workshop_synced_count: workshop_items
            .iter()
            .filter(|entry| {
                matches!(entry.sync_state, wayvid_library::WorkshopSyncState::Synced)
                    && entry.supported_first_release
            })
            .count(),
        monitor_count: 0,
    }
}

#[tauri::command]
pub fn load_app_shell() -> AppShellSnapshot {
    shell_snapshot()
}
