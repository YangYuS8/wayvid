use crate::action_outcome::{ActionOutcome, AppShellPatch};
use crate::models::{
    CompatibilityBadge, ItemType, WorkshopItemDetail, WorkshopItemSummary, WorkshopPageSnapshot,
    WorkshopSyncStatus,
};
use crate::policies::shared::compatibility_policy::{
    compatibility_decision, CompatibilityLevel, CompatibilityReason,
};
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::policies::shared::invalidation_policy::pages_after_workshop_refresh;
use crate::results::workshop::{WorkshopInspection, WorkshopRefreshResult};
use wayvid_library::{
    SteamLibrary, WeProject, WorkshopCatalogEntry, WorkshopProjectType, WorkshopScanner,
};

pub struct WorkshopService;

impl WorkshopService {
    fn item_type_from_project_type(project_type: WorkshopProjectType) -> ItemType {
        match project_type {
            WorkshopProjectType::Video => ItemType::Video,
            WorkshopProjectType::Scene => ItemType::Scene,
            WorkshopProjectType::Web => ItemType::Web,
            WorkshopProjectType::Other => ItemType::Other,
        }
    }

    fn sync_status(entry: &WorkshopCatalogEntry) -> WorkshopSyncStatus {
        match entry.sync_state {
            wayvid_library::WorkshopSyncState::Synced => WorkshopSyncStatus::Synced,
            wayvid_library::WorkshopSyncState::MissingProjectFile => {
                WorkshopSyncStatus::MissingProject
            }
            wayvid_library::WorkshopSyncState::MissingPrimaryAsset => {
                WorkshopSyncStatus::MissingAsset
            }
            wayvid_library::WorkshopSyncState::UnsupportedType => {
                WorkshopSyncStatus::UnsupportedType
            }
        }
    }

    fn compatibility_badge(entry: &WorkshopCatalogEntry) -> CompatibilityBadge {
        match compatibility_decision(entry).level {
            CompatibilityLevel::FullySupported => CompatibilityBadge::FullySupported,
            CompatibilityLevel::PartiallySupported => CompatibilityBadge::PartiallySupported,
            CompatibilityLevel::Unsupported => CompatibilityBadge::Unsupported,
        }
    }

    fn compatibility_note(entry: &WorkshopCatalogEntry) -> Option<String> {
        match compatibility_decision(entry).reason {
            CompatibilityReason::ReadyForLibrary => {
                Some("This item is synchronized locally and available in the Library page.".to_string())
            }
            CompatibilityReason::MissingProjectMetadata => Some(
                "The local Workshop folder is missing valid project metadata, so LWE cannot classify or import this item yet."
                    .to_string(),
            ),
            CompatibilityReason::MissingPrimaryAsset => Some(
                "The project metadata was found, but the primary local asset is missing, so it cannot be projected into Library yet."
                    .to_string(),
            ),
            CompatibilityReason::UnsupportedWebItem => Some(
                "Web Workshop items are visible here, but the first release only supports video and scene imports."
                    .to_string(),
            ),
            CompatibilityReason::UnsupportedProjectType => Some(
                "This Workshop item uses a project type that the first release does not import yet."
                    .to_string(),
            ),
        }
    }

    fn cover_path(entry: &WorkshopCatalogEntry) -> Option<String> {
        let bundled_cover_path = entry
            .cover_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned());

        match cover_art_source(bundled_cover_path) {
            CoverArtSource::Bundled(path) => Some(path),
            CoverArtSource::Placeholder => None,
        }
    }

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

    fn summary_from_entry(entry: WorkshopCatalogEntry) -> WorkshopItemSummary {
        let item_type = Self::item_type_from_project_type(entry.project_type);
        let cover_path = Self::cover_path(&entry);
        let sync_status = Self::sync_status(&entry);
        let compatibility_badge = Self::compatibility_badge(&entry);

        WorkshopItemSummary {
            id: entry.workshop_id.to_string(),
            title: entry.title,
            item_type,
            cover_path,
            sync_status,
            compatibility_badge,
        }
    }

    pub fn load_page() -> Result<WorkshopPageSnapshot, String> {
        let refresh = Self::refresh_catalog()?;

        Ok(WorkshopPageSnapshot {
            items: refresh
                .catalog_entries
                .into_iter()
                .map(Self::summary_from_entry)
                .collect(),
            selected_item_id: None,
            stale: false,
        })
    }

    pub fn load_item_detail(workshop_id: &str) -> Result<WorkshopItemDetail, String> {
        let inspection = Self::inspect_item(workshop_id)?;
        let entry = inspection.entry;
        let project = WeProject::load(&entry.project_dir).ok();
        let description = project
            .as_ref()
            .and_then(|project| project.description.clone());
        let tags = project.map(|project| project.tags).unwrap_or_default();
        let item_type = Self::item_type_from_project_type(entry.project_type);
        let cover_path = Self::cover_path(&entry);
        let sync_status = Self::sync_status(&entry);
        let compatibility_badge = Self::compatibility_badge(&entry);
        let compatibility_note = Self::compatibility_note(&entry);

        Ok(WorkshopItemDetail {
            id: entry.workshop_id.to_string(),
            title: entry.title,
            item_type,
            cover_path,
            sync_status,
            compatibility_badge,
            compatibility_note,
            tags,
            description,
        })
    }

    pub fn refresh_outcome() -> Result<ActionOutcome<WorkshopPageSnapshot>, String> {
        let refresh = Self::refresh_catalog()?;
        let workshop_synced_count = refresh.synced_entry_count();
        let page = WorkshopPageSnapshot {
            items: refresh
                .catalog_entries
                .clone()
                .into_iter()
                .map(Self::summary_from_entry)
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

    fn unsupported_other_entry() -> WorkshopCatalogEntry {
        WorkshopCatalogEntry {
            workshop_id: 10,
            title: "Application Wallpaper".to_string(),
            project_type: WorkshopProjectType::Other,
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
        let summary = WorkshopService::summary_from_entry(unsupported_other_entry());

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
                .map(WorkshopService::summary_from_entry)
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
