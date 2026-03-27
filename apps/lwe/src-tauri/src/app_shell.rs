use crate::library::load_library_projection;
use crate::models::AppShellSnapshot;
use crate::workshop::scan_workshop_catalog;

fn unavailable_app_shell() -> AppShellSnapshot {
    AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: crate::APP_CODE_NAME.to_string(),
        steam_available: false,
        library_count: None,
        workshop_synced_count: None,
        monitor_count: None,
    }
}

fn unavailable_workshop_app_shell() -> AppShellSnapshot {
    AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: crate::APP_CODE_NAME.to_string(),
        steam_available: true,
        library_count: None,
        workshop_synced_count: None,
        monitor_count: None,
    }
}

fn shell_snapshot() -> Result<AppShellSnapshot, String> {
    let steam = match wayvid_library::SteamLibrary::try_discover() {
        Some(steam) => steam,
        None => return Ok(unavailable_app_shell()),
    };

    if !steam.has_wallpaper_engine() {
        return Ok(unavailable_workshop_app_shell());
    }

    let workshop_items = scan_workshop_catalog()?;
    let library_items = load_library_projection()?;

    Ok(AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: crate::APP_CODE_NAME.to_string(),
        steam_available: steam.has_wallpaper_engine(),
        library_count: Some(library_items.len()),
        workshop_synced_count: Some(
            workshop_items
                .iter()
                .filter(|entry| {
                    matches!(entry.sync_state, wayvid_library::WorkshopSyncState::Synced)
                        && entry.supported_first_release
                })
                .count(),
        ),
        monitor_count: None,
    })
}

#[tauri::command]
pub fn load_app_shell() -> Result<AppShellSnapshot, String> {
    shell_snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_steam_leaves_shell_counts_unknown() {
        let snapshot = unavailable_app_shell();

        assert!(!snapshot.steam_available);
        assert_eq!(snapshot.library_count, None);
        assert_eq!(snapshot.workshop_synced_count, None);
        assert_eq!(snapshot.monitor_count, None);
    }

    #[test]
    fn steam_without_wallpaper_engine_content_keeps_counts_unknown() {
        let snapshot = unavailable_workshop_app_shell();

        assert!(snapshot.steam_available);
        assert_eq!(snapshot.library_count, None);
        assert_eq!(snapshot.workshop_synced_count, None);
        assert_eq!(snapshot.monitor_count, None);
    }
}
