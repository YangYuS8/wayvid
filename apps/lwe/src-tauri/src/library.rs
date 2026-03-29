use crate::models::{
    ItemType, LibraryItemDetail, LibraryItemSummary, LibraryPageSnapshot, LibrarySource,
};
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::policies::shared::support_policy::supports_first_release;
use crate::results::library::LibraryProjection;
use crate::services::library_service::LibraryService;
use crate::services::workshop_service::WorkshopService;
use wayvid_library::{WeProject, WorkshopCatalogEntry, WorkshopProjectType};

fn item_type_from_project_type(project_type: WorkshopProjectType) -> ItemType {
    match project_type {
        WorkshopProjectType::Video => ItemType::Video,
        WorkshopProjectType::Scene => ItemType::Scene,
        WorkshopProjectType::Web => ItemType::Web,
        WorkshopProjectType::Other => ItemType::Other,
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

pub(crate) fn library_projection_from_entries(
    entries: Vec<WorkshopCatalogEntry>,
) -> LibraryProjection {
    let source_catalog_count = entries.len();
    let projected_items = entries
        .into_iter()
        .filter(|entry| {
            matches!(entry.sync_state, wayvid_library::WorkshopSyncState::Synced)
                && supports_first_release(entry.project_type)
                && entry.library_item_id.is_some()
        })
        .map(|entry| {
            let cover_path = cover_path(&entry);

            LibraryItemSummary {
                id: entry.library_item_id.unwrap_or_default(),
                title: entry.title,
                item_type: item_type_from_project_type(entry.project_type),
                cover_path,
                source: LibrarySource::Workshop,
                favorite: false,
            }
        })
        .collect();

    LibraryProjection {
        projected_items,
        source_catalog_count,
    }
}

fn detail_from_entry(entry: WorkshopCatalogEntry) -> LibraryItemDetail {
    let project = WeProject::load(&entry.project_dir).ok();
    let description = project
        .as_ref()
        .and_then(|project| project.description.clone());
    let tags = project.map(|project| project.tags).unwrap_or_default();
    let cover_path = cover_path(&entry);

    LibraryItemDetail {
        id: entry.library_item_id.unwrap_or_default(),
        title: entry.title,
        item_type: item_type_from_project_type(entry.project_type),
        cover_path,
        source: LibrarySource::Workshop,
        description,
        tags,
    }
}

#[tauri::command]
pub fn load_library_page() -> Result<LibraryPageSnapshot, String> {
    let projection = LibraryService::load_projection()?;

    Ok(LibraryPageSnapshot {
        items: projection.projected_items,
        selected_item_id: None,
        stale: false,
    })
}

#[tauri::command]
pub fn load_library_item_detail(item_id: String) -> Result<LibraryItemDetail, String> {
    let entry = WorkshopService::refresh_catalog()?
        .catalog_entries
        .into_iter()
        .find(|entry| {
            matches!(entry.sync_state, wayvid_library::WorkshopSyncState::Synced)
                && supports_first_release(entry.project_type)
                && entry.library_item_id.as_deref() == Some(item_id.as_str())
        })
        .ok_or_else(|| format!("Library item {item_id} not found"))?;

    Ok(detail_from_entry(entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wayvid_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

    #[test]
    fn shared_policy_library_projection_uses_result_type() {
        let projection = library_projection_from_entries(vec![WorkshopCatalogEntry {
            workshop_id: 7,
            title: "Synced Scene".to_string(),
            project_type: WorkshopProjectType::Scene,
            project_dir: std::path::PathBuf::from("/tmp/7"),
            cover_path: None,
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("scene-7".to_string()),
        }]);

        assert_eq!(projection.source_catalog_count, 1);
        assert_eq!(projection.projected_items.len(), 1);
    }
}
