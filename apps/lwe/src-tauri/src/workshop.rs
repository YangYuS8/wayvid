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
use crate::services::workshop_service::WorkshopService;
use wayvid_library::{
    SteamLibrary, WeProject, WorkshopCatalogEntry, WorkshopProjectType, WorkshopScanner,
};

pub(crate) fn workshop_item_url(workshop_id: &str) -> String {
    format!("https://steamcommunity.com/sharedfiles/filedetails/?id={workshop_id}")
}

pub(crate) fn steam_openurl(workshop_id: &str) -> String {
    format!("steam://openurl/{}", workshop_item_url(workshop_id))
}

pub(crate) fn scan_workshop_catalog() -> Result<Vec<WorkshopCatalogEntry>, String> {
    let steam = SteamLibrary::discover()
        .map_err(|error| format!("Steam Workshop is unavailable: {error}"))?;
    if !steam.has_wallpaper_engine() {
        return Err("Wallpaper Engine Workshop content is unavailable on this machine".to_string());
    }

    let mut scanner = WorkshopScanner::new(steam);

    scanner
        .scan_catalog()
        .map_err(|error| format!("Failed to scan the Steam Workshop catalog: {error}"))
}

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
        wayvid_library::WorkshopSyncState::MissingProjectFile => WorkshopSyncStatus::MissingProject,
        wayvid_library::WorkshopSyncState::MissingPrimaryAsset => WorkshopSyncStatus::MissingAsset,
        wayvid_library::WorkshopSyncState::UnsupportedType => WorkshopSyncStatus::UnsupportedType,
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

pub(crate) fn workshop_refresh_result(
    catalog_entries: Vec<WorkshopCatalogEntry>,
) -> WorkshopRefreshResult {
    WorkshopRefreshResult {
        catalog_entries,
        library_refresh_required: true,
    }
}

fn refresh_outcome_from_refresh_result(
    refresh: WorkshopRefreshResult,
) -> Result<ActionOutcome<WorkshopPageSnapshot>, String> {
    let workshop_synced_count = refresh.synced_entry_count();
    let page = WorkshopPageSnapshot {
        items: refresh
            .catalog_entries
            .clone()
            .into_iter()
            .map(summary_from_entry)
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

fn summary_from_entry(entry: WorkshopCatalogEntry) -> WorkshopItemSummary {
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = cover_path(&entry);
    let sync_status = sync_status(&entry);
    let compatibility_badge = compatibility_badge(&entry);

    WorkshopItemSummary {
        id: entry.workshop_id.to_string(),
        title: entry.title,
        item_type,
        cover_path,
        sync_status,
        compatibility_badge,
    }
}

fn detail_from_entry(entry: WorkshopCatalogEntry) -> WorkshopItemDetail {
    let project = WeProject::load(&entry.project_dir).ok();
    let description = project
        .as_ref()
        .and_then(|project| project.description.clone());
    let tags = project.map(|project| project.tags).unwrap_or_default();
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = cover_path(&entry);
    let sync_status = sync_status(&entry);
    let compatibility_badge = compatibility_badge(&entry);
    let compatibility_note = compatibility_note(&entry);

    WorkshopItemDetail {
        id: entry.workshop_id.to_string(),
        title: entry.title,
        item_type,
        cover_path,
        sync_status,
        compatibility_badge,
        compatibility_note,
        tags,
        description,
    }
}

fn detail_from_inspection(inspection: WorkshopInspection) -> WorkshopItemDetail {
    detail_from_entry(inspection.entry)
}

fn workshop_page_from_refresh_result(refresh: WorkshopRefreshResult) -> WorkshopPageSnapshot {
    WorkshopPageSnapshot {
        items: refresh
            .catalog_entries
            .into_iter()
            .map(summary_from_entry)
            .collect(),
        selected_item_id: None,
        stale: false,
    }
}

#[tauri::command]
pub fn load_workshop_page() -> Result<WorkshopPageSnapshot, String> {
    Ok(workshop_page_from_refresh_result(
        WorkshopService::refresh_catalog()?,
    ))
}

#[tauri::command]
pub fn load_workshop_item_detail(workshop_id: String) -> Result<WorkshopItemDetail, String> {
    Ok(detail_from_inspection(WorkshopService::inspect_item(
        &workshop_id,
    )?))
}

#[tauri::command]
pub fn refresh_workshop_catalog() -> Result<ActionOutcome<WorkshopPageSnapshot>, String> {
    refresh_outcome_from_refresh_result(WorkshopService::refresh_catalog()?)
}

#[tauri::command]
pub fn open_workshop_in_steam(workshop_id: String) -> Result<ActionOutcome<()>, String> {
    open::that_detached(steam_openurl(&workshop_id)).map_err(|error| error.to_string())?;

    Ok(ActionOutcome {
        ok: true,
        message: Some("Opened item in Steam".to_string()),
        shell_patch: None,
        current_update: None,
        invalidations: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wayvid_library::WorkshopSyncState;

    #[test]
    fn steam_url_uses_official_workshop_page() {
        assert_eq!(
            workshop_item_url("12345"),
            "https://steamcommunity.com/sharedfiles/filedetails/?id=12345"
        );
    }

    #[test]
    fn steam_openurl_wraps_official_workshop_page() {
        assert_eq!(
            steam_openurl("12345"),
            "steam://openurl/https://steamcommunity.com/sharedfiles/filedetails/?id=12345"
        );
    }

    #[test]
    fn missing_project_metadata_is_distinct_from_unsupported_type() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 9,
            title: "Broken Item".to_string(),
            project_type: WorkshopProjectType::Other,
            project_dir: std::path::PathBuf::from("/tmp/9"),
            cover_path: None,
            sync_state: WorkshopSyncState::MissingProjectFile,
            supported_first_release: false,
            library_item_id: None,
        };

        assert_eq!(sync_status(&entry), WorkshopSyncStatus::MissingProject);
        assert_eq!(compatibility_badge(&entry), CompatibilityBadge::Unsupported);
        assert!(compatibility_note(&entry).unwrap().contains("metadata"));
    }

    #[test]
    fn unsupported_other_type_uses_unsupported_badge() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 10,
            title: "Application Wallpaper".to_string(),
            project_type: WorkshopProjectType::Other,
            project_dir: std::path::PathBuf::from("/tmp/10"),
            cover_path: None,
            sync_state: WorkshopSyncState::UnsupportedType,
            supported_first_release: false,
            library_item_id: None,
        };

        assert_eq!(sync_status(&entry), WorkshopSyncStatus::UnsupportedType);
        assert_eq!(compatibility_badge(&entry), CompatibilityBadge::Unsupported);
    }

    #[test]
    fn shared_policy_workshop_refresh_result_drives_outcome_counts() {
        let refresh = workshop_refresh_result(vec![WorkshopCatalogEntry {
            workshop_id: 11,
            title: "Synced Scene".to_string(),
            project_type: WorkshopProjectType::Scene,
            project_dir: std::path::PathBuf::from("/tmp/11"),
            cover_path: None,
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("scene-11".to_string()),
        }]);

        assert!(refresh.library_refresh_required);
        assert_eq!(refresh.synced_entry_count(), 1);
    }

    #[test]
    fn shared_policy_refresh_outcome_uses_result_invalidation_effect() {
        let outcome = refresh_outcome_from_refresh_result(WorkshopRefreshResult {
            catalog_entries: Vec::new(),
            library_refresh_required: false,
        })
        .unwrap();

        assert!(outcome.invalidations.is_empty());
    }
}
