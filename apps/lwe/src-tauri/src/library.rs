use crate::models::{ItemType, LibraryItemDetail, LibraryPageSnapshot, LibrarySource};
use crate::services::library_service::LibraryService;
use wayvid_library::{WeProject, WorkshopCatalogEntry, WorkshopProjectType};

fn item_type_from_project_type(project_type: WorkshopProjectType) -> ItemType {
    match project_type {
        WorkshopProjectType::Video => ItemType::Video,
        WorkshopProjectType::Scene => ItemType::Scene,
        WorkshopProjectType::Web => ItemType::Web,
        WorkshopProjectType::Other => ItemType::Other,
    }
}

fn detail_from_entry(entry: WorkshopCatalogEntry) -> LibraryItemDetail {
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = entry
        .cover_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .filter(|path| !path.trim().is_empty());
    let project = WeProject::load(&entry.project_dir).ok();
    let description = project
        .as_ref()
        .and_then(|project| project.description.clone());
    let tags = project.map(|project| project.tags).unwrap_or_default();

    LibraryItemDetail {
        id: entry.library_item_id.unwrap_or_default(),
        title: entry.title,
        item_type,
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
    let entry = LibraryService::inspect_item(&item_id)?;

    Ok(detail_from_entry(entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wayvid_library::WorkshopProjectType;

    #[test]
    fn detail_from_entry_maps_project_type_to_item_type() {
        let detail = detail_from_entry(WorkshopCatalogEntry {
            workshop_id: 7,
            title: "Synced Scene".to_string(),
            project_type: WorkshopProjectType::Scene,
            project_dir: std::path::PathBuf::from("/tmp/7"),
            cover_path: None,
            sync_state: wayvid_library::WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("scene-7".to_string()),
        });

        assert_eq!(detail.item_type, ItemType::Scene);
        assert_eq!(detail.id, "scene-7");
    }
}
