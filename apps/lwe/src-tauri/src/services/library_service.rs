use crate::models::{LibraryItemDetail, LibraryPageSnapshot};
use crate::results::library::LibraryProjection;
use crate::results::workshop::WorkshopRefreshResult;
use crate::services::catalog_mapper::{
    includes_library_item, library_detail_from_entry, library_summary_from_entry,
};
use crate::services::workshop_service::WorkshopService;
use wayvid_library::WorkshopCatalogEntry;

pub struct LibraryService;

impl LibraryService {
    pub fn projection_from_refresh(refresh: WorkshopRefreshResult) -> LibraryProjection {
        let source_catalog_count = refresh.catalog_entries.len();
        let projected_items = refresh
            .catalog_entries
            .into_iter()
            .filter(includes_library_item)
            .map(library_summary_from_entry)
            .collect();

        LibraryProjection {
            projected_items,
            source_catalog_count,
        }
    }

    pub fn load_projection() -> Result<LibraryProjection, String> {
        Ok(Self::projection_from_refresh(
            WorkshopService::refresh_catalog()?,
        ))
    }

    pub fn inspect_item(item_id: &str) -> Result<WorkshopCatalogEntry, String> {
        WorkshopService::refresh_catalog()?
            .catalog_entries
            .into_iter()
            .find(|entry| {
                includes_library_item(entry) && entry.library_item_id.as_deref() == Some(item_id)
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
        Ok(library_detail_from_entry(Self::inspect_item(item_id)?))
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
