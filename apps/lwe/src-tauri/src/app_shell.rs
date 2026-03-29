use crate::models::AppShellSnapshot;
use crate::results::app_shell::{ObservedCount, ShellSummary};
use crate::services::library_service::LibraryService;
use crate::services::workshop_service::WorkshopService;
use wayvid_library::WorkshopCatalogEntry;

fn observed_count_to_option(count: ObservedCount) -> Option<usize> {
    match count {
        ObservedCount::Known(value) => Some(value),
        ObservedCount::Unknown => None,
    }
}

fn shell_snapshot_from_summary(summary: ShellSummary) -> AppShellSnapshot {
    AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: crate::APP_CODE_NAME.to_string(),
        steam_available: summary.steam_available,
        library_count: observed_count_to_option(summary.library_items),
        workshop_synced_count: observed_count_to_option(summary.synced_workshop_items),
        monitor_count: observed_count_to_option(summary.connected_monitors),
    }
}

fn unavailable_app_shell() -> AppShellSnapshot {
    shell_snapshot_from_summary(ShellSummary {
        steam_available: false,
        library_items: ObservedCount::Unknown,
        synced_workshop_items: ObservedCount::Unknown,
        connected_monitors: ObservedCount::Unknown,
    })
}

fn unavailable_workshop_app_shell() -> AppShellSnapshot {
    shell_snapshot_from_summary(ShellSummary {
        steam_available: true,
        library_items: ObservedCount::Unknown,
        synced_workshop_items: ObservedCount::Unknown,
        connected_monitors: ObservedCount::Unknown,
    })
}

fn shell_summary_from_catalog(
    steam_available: bool,
    workshop_items: Vec<WorkshopCatalogEntry>,
) -> ShellSummary {
    let synced_workshop_items = workshop_items
        .iter()
        .filter(|entry| entry.sync_state == wayvid_library::WorkshopSyncState::Synced)
        .count();
    let source_catalog_count = workshop_items.len();
    let projected_items = workshop_items
        .into_iter()
        .filter(|entry| {
            matches!(entry.sync_state, wayvid_library::WorkshopSyncState::Synced)
                && matches!(
                    entry.project_type,
                    wayvid_library::WorkshopProjectType::Video
                        | wayvid_library::WorkshopProjectType::Scene
                )
                && entry.library_item_id.is_some()
        })
        .count();

    ShellSummary {
        steam_available,
        library_items: ObservedCount::Known(projected_items.min(source_catalog_count)),
        synced_workshop_items: ObservedCount::Known(synced_workshop_items),
        connected_monitors: ObservedCount::Unknown,
    }
}

fn shell_snapshot_from_catalog(
    steam_available: bool,
    workshop_items: Vec<WorkshopCatalogEntry>,
) -> AppShellSnapshot {
    shell_snapshot_from_summary(shell_summary_from_catalog(steam_available, workshop_items))
}

fn shell_snapshot() -> Result<AppShellSnapshot, String> {
    let steam = match wayvid_library::SteamLibrary::try_discover() {
        Some(steam) => steam,
        None => return Ok(unavailable_app_shell()),
    };

    if !steam.has_wallpaper_engine() {
        return Ok(unavailable_workshop_app_shell());
    }

    let refresh = WorkshopService::refresh_catalog()?;
    let library_projection = LibraryService::load_projection()?;

    Ok(shell_snapshot_from_summary(ShellSummary {
        steam_available: steam.has_wallpaper_engine(),
        library_items: ObservedCount::Known(library_projection.projected_items.len()),
        synced_workshop_items: ObservedCount::Known(refresh.synced_entry_count()),
        connected_monitors: ObservedCount::Unknown,
    }))
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

    #[test]
    fn shared_policy_shell_snapshot_comes_from_result_summary() {
        let summary = shell_summary_from_catalog(
            true,
            vec![WorkshopCatalogEntry {
                workshop_id: 1,
                title: "Synced Scene".to_string(),
                project_type: WorkshopProjectType::Scene,
                project_dir: std::path::PathBuf::from("/tmp/1"),
                cover_path: None,
                sync_state: WorkshopSyncState::Synced,
                supported_first_release: true,
                library_item_id: Some("scene-1".to_string()),
            }],
        );

        assert!(matches!(
            summary.library_items,
            crate::results::app_shell::ObservedCount::Known(1)
        ));
        assert!(matches!(
            summary.synced_workshop_items,
            crate::results::app_shell::ObservedCount::Known(1)
        ));
    }
}
