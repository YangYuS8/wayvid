use crate::assembly::app_shell::assemble_app_shell;
use crate::models::AppShellSnapshot;
use crate::results::app_shell::{ObservedCount, ShellSummary};
use crate::services::library_service::LibraryService;
use crate::services::workshop_service::WorkshopService;

fn unavailable_app_shell_summary() -> ShellSummary {
    ShellSummary {
        steam_available: false,
        library_items: ObservedCount::Unknown,
        synced_workshop_items: ObservedCount::Unknown,
        connected_monitors: ObservedCount::Unknown,
    }
}

fn unavailable_workshop_app_shell_summary() -> ShellSummary {
    ShellSummary {
        steam_available: true,
        library_items: ObservedCount::Unknown,
        synced_workshop_items: ObservedCount::Unknown,
        connected_monitors: ObservedCount::Unknown,
    }
}

fn shell_summary_from_refresh(
    steam_available: bool,
    refresh: crate::results::workshop::WorkshopRefreshResult,
) -> ShellSummary {
    let synced_workshop_items = refresh.synced_entry_count();
    let library_projection = LibraryService::projection_from_refresh(refresh);

    ShellSummary {
        steam_available,
        library_items: ObservedCount::Known(library_projection.projected_items.len()),
        synced_workshop_items: ObservedCount::Known(synced_workshop_items),
        connected_monitors: ObservedCount::Unknown,
    }
}

fn shell_summary() -> Result<ShellSummary, String> {
    let steam = match wayvid_library::SteamLibrary::try_discover() {
        Some(steam) => steam,
        None => return Ok(unavailable_app_shell_summary()),
    };

    if !steam.has_wallpaper_engine() {
        return Ok(unavailable_workshop_app_shell_summary());
    }

    Ok(shell_summary_from_refresh(
        steam.has_wallpaper_engine(),
        WorkshopService::refresh_catalog()?,
    ))
}

#[tauri::command]
pub fn load_app_shell() -> Result<AppShellSnapshot, String> {
    Ok(assemble_app_shell(shell_summary()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::workshop::WorkshopRefreshResult;
    use wayvid_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

    #[test]
    fn unavailable_steam_leaves_shell_counts_unknown() {
        let snapshot = assemble_app_shell(unavailable_app_shell_summary());

        assert!(!snapshot.steam_available);
        assert_eq!(snapshot.library_count, None);
        assert_eq!(snapshot.workshop_synced_count, None);
        assert_eq!(snapshot.monitor_count, None);
    }

    #[test]
    fn steam_without_wallpaper_engine_content_keeps_counts_unknown() {
        let snapshot = assemble_app_shell(unavailable_workshop_app_shell_summary());

        assert!(snapshot.steam_available);
        assert_eq!(snapshot.library_count, None);
        assert_eq!(snapshot.workshop_synced_count, None);
        assert_eq!(snapshot.monitor_count, None);
    }

    #[test]
    fn shell_counts_come_from_one_refresh_result() {
        let snapshot = assemble_app_shell(shell_summary_from_refresh(
            true,
            WorkshopRefreshResult {
                catalog_entries: vec![
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
                library_refresh_required: true,
            },
        ));

        assert_eq!(snapshot.library_count, Some(1));
        assert_eq!(snapshot.workshop_synced_count, Some(1));
    }

    #[test]
    fn shared_policy_shell_snapshot_comes_from_result_summary() {
        let summary = shell_summary_from_refresh(
            true,
            WorkshopRefreshResult {
                catalog_entries: vec![WorkshopCatalogEntry {
                    workshop_id: 1,
                    title: "Synced Scene".to_string(),
                    project_type: WorkshopProjectType::Scene,
                    project_dir: std::path::PathBuf::from("/tmp/1"),
                    cover_path: None,
                    sync_state: WorkshopSyncState::Synced,
                    supported_first_release: true,
                    library_item_id: Some("scene-1".to_string()),
                }],
                library_refresh_required: true,
            },
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
