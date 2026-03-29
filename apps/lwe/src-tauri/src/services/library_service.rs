use crate::models::{ItemType, LibraryItemSummary, LibrarySource};
use crate::models::{LibraryItemDetail, LibraryPageSnapshot};
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::policies::shared::support_policy::supports_first_release;
use crate::results::library::LibraryProjection;
use crate::services::workshop_service::WorkshopService;
use wayvid_library::{WeProject, WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

pub struct LibraryService;

impl LibraryService {
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

    fn includes_library_item(entry: &WorkshopCatalogEntry) -> bool {
        matches!(entry.sync_state, WorkshopSyncState::Synced)
            && supports_first_release(entry.project_type)
            && entry.library_item_id.is_some()
    }

    pub fn load_projection() -> Result<LibraryProjection, String> {
        let refresh = WorkshopService::refresh_catalog()?;
        let source_catalog_count = refresh.catalog_entries.len();
        let projected_items = refresh
            .catalog_entries
            .into_iter()
            .filter(Self::includes_library_item)
            .map(|entry| {
                let item_type = Self::item_type_from_project_type(entry.project_type);
                let cover_path = Self::cover_path(&entry);

                LibraryItemSummary {
                    id: entry.library_item_id.unwrap_or_default(),
                    title: entry.title,
                    item_type,
                    cover_path,
                    source: LibrarySource::Workshop,
                    favorite: false,
                }
            })
            .collect();

        Ok(LibraryProjection {
            projected_items,
            source_catalog_count,
        })
    }

    pub fn inspect_item(item_id: &str) -> Result<WorkshopCatalogEntry, String> {
        WorkshopService::refresh_catalog()?
            .catalog_entries
            .into_iter()
            .find(|entry| {
                Self::includes_library_item(entry)
                    && entry.library_item_id.as_deref() == Some(item_id)
            })
            .ok_or_else(|| format!("Library item {item_id} not found"))
    }

    pub fn load_page() -> Result<LibraryPageSnapshot, String> {
        let projection = Self::load_projection()?;

        Ok(LibraryPageSnapshot {
            items: projection.projected_items,
            selected_item_id: None,
            stale: false,
        })
    }

    pub fn load_item_detail(item_id: &str) -> Result<LibraryItemDetail, String> {
        let entry = Self::inspect_item(item_id)?;
        let item_type = Self::item_type_from_project_type(entry.project_type);
        let cover_path = Self::cover_path(&entry);
        let project = WeProject::load(&entry.project_dir).ok();
        let description = project
            .as_ref()
            .and_then(|project| project.description.clone());
        let tags = project.map(|project| project.tags).unwrap_or_default();

        Ok(LibraryItemDetail {
            id: entry.library_item_id.unwrap_or_default(),
            title: entry.title,
            item_type,
            cover_path,
            source: LibrarySource::Workshop,
            description,
            tags,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::results::library::LibraryProjection;

    #[test]
    fn service_layer_library_service_uses_application_projection_result() {
        let result = LibraryProjection {
            projected_items: Vec::new(),
            source_catalog_count: 0,
        };

        assert!(result.projected_items.is_empty());
        assert_eq!(result.source_catalog_count, 0);
    }
}
