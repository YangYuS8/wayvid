use crate::library::project_library_items;
use crate::models::AppShellSnapshot;
use crate::workshop::scan_workshop_catalog;
use wayvid_library::WorkshopCatalogEntry;

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

fn shell_snapshot_from_catalog(
    steam_available: bool,
    workshop_items: Vec<WorkshopCatalogEntry>,
) -> AppShellSnapshot {
    let workshop_synced_count = workshop_items
        .iter()
        .filter(|entry| {
            matches!(entry.sync_state, wayvid_library::WorkshopSyncState::Synced)
                && entry.supported_first_release
        })
        .count();
    let library_count = project_library_items(workshop_items).len();

    AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: crate::APP_CODE_NAME.to_string(),
        steam_available,
        library_count: Some(library_count),
        workshop_synced_count: Some(workshop_synced_count),
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

    Ok(shell_snapshot_from_catalog(
        steam.has_wallpaper_engine(),
        workshop_items,
    ))
}

#[tauri::command]
pub fn load_app_shell() -> Result<AppShellSnapshot, String> {
    shell_snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wayvid_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

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

    #[test]
    fn shell_counts_come_from_one_catalog_view() {
        let snapshot = shell_snapshot_from_catalog(
            true,
            vec![
                WorkshopCatalogEntry {
                    workshop_id: 1,
                    title: "Synced Scene".to_string(),
                    project_type: WorkshopProjectType::Scene,
                    project_dir: std::path::PathBuf::from("/tmp/1"),
                    cover_path: None,
                    sync_state: WorkshopSyncState::Synced,
                    supported_first_release: true,
                    library_item_id: Some("scene-1".to_string()),
                },
                WorkshopCatalogEntry {
                    workshop_id: 2,
                    title: "Unsupported App".to_string(),
                    project_type: WorkshopProjectType::Other,
                    project_dir: std::path::PathBuf::from("/tmp/2"),
                    cover_path: None,
                    sync_state: WorkshopSyncState::UnsupportedType,
                    supported_first_release: false,
                    library_item_id: None,
                },
            ],
        );

        assert_eq!(snapshot.library_count, Some(1));
        assert_eq!(snapshot.workshop_synced_count, Some(1));
    }
}
