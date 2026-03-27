use crate::models::{
    ItemType, LibraryItemDetail, LibraryItemSummary, LibraryPageSnapshot, LibrarySource,
};
use wayvid_library::{WeProject, WorkshopCatalogEntry, WorkshopProjectType, WorkshopScanner};

fn item_type_from_project_type(project_type: WorkshopProjectType) -> ItemType {
    match project_type {
        WorkshopProjectType::Video => ItemType::Video,
        WorkshopProjectType::Scene => ItemType::Scene,
        WorkshopProjectType::Web => ItemType::Web,
        WorkshopProjectType::Other => ItemType::Other,
    }
}

pub(crate) fn load_library_projection() -> Vec<LibraryItemSummary> {
    let mut scanner = match WorkshopScanner::try_discover() {
        Some(scanner) => scanner,
        None => return Vec::new(),
    };

    let Ok(entries) = scanner.scan_catalog() else {
        return Vec::new();
    };

    entries
        .into_iter()
        .filter(|entry| {
            matches!(entry.sync_state, wayvid_library::WorkshopSyncState::Synced)
                && entry.supported_first_release
                && entry.library_item_id.is_some()
        })
        .map(|entry| LibraryItemSummary {
            id: entry.library_item_id.unwrap_or_default(),
            title: entry.title,
            item_type: item_type_from_project_type(entry.project_type),
            cover_path: entry
                .cover_path
                .map(|path| path.to_string_lossy().into_owned()),
            source: LibrarySource::Workshop,
            favorite: false,
        })
        .collect()
}

fn detail_from_entry(entry: WorkshopCatalogEntry) -> LibraryItemDetail {
    let project = WeProject::load(&entry.project_dir).ok();
    let description = project
        .as_ref()
        .and_then(|project| project.description.clone());
    let tags = project.map(|project| project.tags).unwrap_or_default();

    LibraryItemDetail {
        id: entry.library_item_id.unwrap_or_default(),
        title: entry.title,
        item_type: item_type_from_project_type(entry.project_type),
        cover_path: entry
            .cover_path
            .map(|path| path.to_string_lossy().into_owned()),
        source: LibrarySource::Workshop,
        description,
        tags,
    }
}

#[tauri::command]
pub fn load_library_page() -> LibraryPageSnapshot {
    LibraryPageSnapshot {
        items: load_library_projection(),
        selected_item_id: None,
        stale: false,
    }
}

#[tauri::command]
pub fn load_library_item_detail(item_id: String) -> Result<LibraryItemDetail, String> {
    let mut scanner = WorkshopScanner::try_discover()
        .ok_or_else(|| "Steam Workshop is not available on this machine".to_string())?;

    let entry = scanner
        .scan_catalog()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|entry| {
            matches!(entry.sync_state, wayvid_library::WorkshopSyncState::Synced)
                && entry.supported_first_release
                && entry.library_item_id.as_deref() == Some(item_id.as_str())
        })
        .ok_or_else(|| format!("Library item {item_id} not found"))?;

    Ok(detail_from_entry(entry))
}
