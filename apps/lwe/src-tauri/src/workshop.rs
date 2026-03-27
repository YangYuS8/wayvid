use crate::action_outcome::{ActionOutcome, AppShellPatch, InvalidatedPage};
use crate::models::{
    CompatibilityBadge, ItemType, WorkshopItemDetail, WorkshopItemSummary, WorkshopPageSnapshot,
    WorkshopSyncStatus,
};
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
    if entry.supported_first_release {
        CompatibilityBadge::FullySupported
    } else if matches!(
        entry.sync_state,
        wayvid_library::WorkshopSyncState::MissingProjectFile
    ) || matches!(entry.project_type, WorkshopProjectType::Web)
    {
        CompatibilityBadge::Unsupported
    } else {
        CompatibilityBadge::PartiallySupported
    }
}

fn compatibility_note(entry: &WorkshopCatalogEntry) -> Option<String> {
    if entry.supported_first_release {
        Some("This item is synchronized locally and available in the Library page.".to_string())
    } else {
        if matches!(
            entry.sync_state,
            wayvid_library::WorkshopSyncState::MissingProjectFile
        ) {
            return Some(
                "The local Workshop folder is missing valid project metadata, so LWE cannot classify or import this item yet."
                    .to_string(),
            );
        }

        match entry.project_type {
            WorkshopProjectType::Web => Some(
                "Web Workshop items are visible here, but the first release only supports video and scene imports."
                    .to_string(),
            ),
            WorkshopProjectType::Other => Some(
                "This Workshop item uses a project type that the first release does not import yet."
                    .to_string(),
            ),
            WorkshopProjectType::Video | WorkshopProjectType::Scene => Some(
                "The project metadata was found, but the primary local asset is missing, so it cannot be projected into Library yet."
                    .to_string(),
            ),
        }
    }
}

fn summary_from_entry(entry: WorkshopCatalogEntry) -> WorkshopItemSummary {
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = entry
        .cover_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());
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
    let cover_path = entry
        .cover_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());
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

fn workshop_page_from_scan_result(
    scan_result: Result<Vec<WorkshopCatalogEntry>, String>,
) -> Result<WorkshopPageSnapshot, String> {
    Ok(WorkshopPageSnapshot {
        items: scan_result?.into_iter().map(summary_from_entry).collect(),
        selected_item_id: None,
        stale: false,
    })
}

fn refresh_outcome_from_scan_result(
    scan_result: Result<Vec<WorkshopCatalogEntry>, String>,
) -> Result<ActionOutcome<WorkshopPageSnapshot>, String> {
    let page = workshop_page_from_scan_result(scan_result)?;
    let workshop_synced_count = page
        .items
        .iter()
        .filter(|item| item.sync_status == WorkshopSyncStatus::Synced)
        .count();

    Ok(ActionOutcome {
        ok: true,
        message: Some("Workshop catalog refreshed".to_string()),
        shell_patch: Some(AppShellPatch {
            workshop_synced_count: Some(workshop_synced_count),
            library_count: None,
            monitor_count: None,
        }),
        current_update: Some(page),
        invalidations: vec![InvalidatedPage::Library],
    })
}

#[tauri::command]
pub fn load_workshop_page() -> Result<WorkshopPageSnapshot, String> {
    workshop_page_from_scan_result(scan_workshop_catalog())
}

#[tauri::command]
pub fn load_workshop_item_detail(workshop_id: String) -> Result<WorkshopItemDetail, String> {
    let entry = scan_workshop_catalog()?
        .into_iter()
        .find(|entry| entry.workshop_id.to_string() == workshop_id)
        .ok_or_else(|| format!("Workshop item {workshop_id} not found"))?;

    Ok(detail_from_entry(entry))
}

#[tauri::command]
pub fn refresh_workshop_catalog() -> Result<ActionOutcome<WorkshopPageSnapshot>, String> {
    refresh_outcome_from_scan_result(scan_workshop_catalog())
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
    fn workshop_page_from_scan_result_propagates_failures() {
        let error = workshop_page_from_scan_result(Err("scan failed".to_string())).unwrap_err();

        assert_eq!(error, "scan failed");
    }

    #[test]
    fn refresh_outcome_from_scan_result_propagates_failures() {
        let error = refresh_outcome_from_scan_result(Err("scan failed".to_string())).unwrap_err();

        assert_eq!(error, "scan failed");
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
}
