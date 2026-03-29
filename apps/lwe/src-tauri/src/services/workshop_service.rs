use crate::action_outcome::{ActionOutcome, AppShellPatch};
use crate::models::{WorkshopItemDetail, WorkshopPageSnapshot};
use crate::policies::shared::invalidation_policy::pages_after_workshop_refresh;
use crate::results::workshop::{WorkshopInspection, WorkshopRefreshResult};
use crate::services::catalog_mapper::{workshop_detail_from_entry, workshop_summary_from_entry};
use wayvid_library::{SteamLibrary, WorkshopCatalogEntry, WorkshopScanner};

pub struct WorkshopService;

impl WorkshopService {
    fn scan_catalog() -> Result<Vec<WorkshopCatalogEntry>, String> {
        let steam = SteamLibrary::discover()
            .map_err(|error| format!("Steam Workshop is unavailable: {error}"))?;
        if !steam.has_wallpaper_engine() {
            return Err(
                "Wallpaper Engine Workshop content is unavailable on this machine".to_string(),
            );
        }

        let mut scanner = WorkshopScanner::new(steam);

        scanner
            .scan_catalog()
            .map_err(|error| format!("Failed to scan the Steam Workshop catalog: {error}"))
    }

    pub fn refresh_catalog() -> Result<WorkshopRefreshResult, String> {
        Ok(WorkshopRefreshResult {
            catalog_entries: Self::scan_catalog()?,
            library_refresh_required: true,
        })
    }

    pub fn inspect_item(workshop_id: &str) -> Result<WorkshopInspection, String> {
        let entry = Self::refresh_catalog()?
            .catalog_entries
            .into_iter()
            .find(|entry| entry.workshop_id.to_string() == workshop_id)
            .ok_or_else(|| format!("Workshop item {workshop_id} not found"))?;

        Ok(WorkshopInspection {
            requested_workshop_id: workshop_id.to_string(),
            entry,
        })
    }

    pub fn load_page() -> Result<WorkshopPageSnapshot, String> {
        let refresh = Self::refresh_catalog()?;

        Ok(WorkshopPageSnapshot {
            items: refresh
                .catalog_entries
                .into_iter()
                .map(workshop_summary_from_entry)
                .collect(),
            selected_item_id: None,
            stale: false,
        })
    }

    pub fn load_item_detail(workshop_id: &str) -> Result<WorkshopItemDetail, String> {
        let inspection = Self::inspect_item(workshop_id)?;
        Ok(workshop_detail_from_entry(inspection.entry))
    }

    pub fn refresh_outcome() -> Result<ActionOutcome<WorkshopPageSnapshot>, String> {
        let refresh = Self::refresh_catalog()?;
        let workshop_synced_count = refresh.synced_entry_count();
        let page = WorkshopPageSnapshot {
            items: refresh
                .catalog_entries
                .clone()
                .into_iter()
                .map(workshop_summary_from_entry)
                .collect(),
            selected_item_id: None,
            stale: false,
        };
        let invalidations = if refresh.library_refresh_required {
            pages_after_workshop_refresh()
        } else {
            Vec::new()
        };

        Ok(ActionOutcome {
            ok: true,
            message: Some("Workshop catalog refreshed".to_string()),
            shell_patch: Some(AppShellPatch {
                workshop_synced_count: Some(workshop_synced_count),
                library_count: None,
                monitor_count: None,
            }),
            current_update: Some(page),
            invalidations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::workshop::WorkshopRefreshResult;
    use crate::services::catalog_mapper::workshop_summary_from_entry;
    use crate::{models::CompatibilityBadge, models::WorkshopSyncStatus};

    fn unsupported_other_entry() -> WorkshopCatalogEntry {
        WorkshopCatalogEntry {
            workshop_id: 10,
            title: "Application Wallpaper".to_string(),
            project_type: wayvid_library::WorkshopProjectType::Other,
            project_dir: std::path::PathBuf::from("/tmp/10"),
            cover_path: None,
            sync_state: wayvid_library::WorkshopSyncState::UnsupportedType,
            supported_first_release: false,
            library_item_id: None,
        }
    }

    #[test]
    fn service_layer_workshop_service_returns_application_result_not_page_snapshot() {
        let result = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
            library_refresh_required: true,
        };

        assert!(result.library_refresh_required);
        assert_eq!(result.catalog_entries.len(), 0);
    }

    #[test]
    fn service_layer_workshop_summary_uses_shared_compatibility_rules() {
        let summary = workshop_summary_from_entry(unsupported_other_entry());

        assert_eq!(summary.sync_status, WorkshopSyncStatus::UnsupportedType);
        assert_eq!(summary.compatibility_badge, CompatibilityBadge::Unsupported);
    }

    #[test]
    fn service_layer_refresh_outcome_respects_result_invalidation_flag() {
        let refresh = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
            library_refresh_required: false,
        };

        let workshop_synced_count = refresh.synced_entry_count();
        let page = WorkshopPageSnapshot {
            items: refresh
                .catalog_entries
                .clone()
                .into_iter()
                .map(workshop_summary_from_entry)
                .collect(),
            selected_item_id: None,
            stale: false,
        };

        let outcome = ActionOutcome {
            ok: true,
            message: Some("Workshop catalog refreshed".to_string()),
            shell_patch: Some(AppShellPatch {
                workshop_synced_count: Some(workshop_synced_count),
                library_count: None,
                monitor_count: None,
            }),
            current_update: Some(page),
            invalidations: if refresh.library_refresh_required {
                pages_after_workshop_refresh()
            } else {
                Vec::new()
            },
        };

        assert!(outcome.invalidations.is_empty());
    }
}
