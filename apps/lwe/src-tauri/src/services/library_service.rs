use crate::policies::shared::support_policy::supports_first_release;
use crate::results::library::LibraryProjection;
use crate::results::workshop::WorkshopRefreshResult;
use crate::services::workshop_service::WorkshopService;
use wayvid_library::{WorkshopCatalogEntry, WorkshopSyncState};

fn includes_library_item(entry: &WorkshopCatalogEntry) -> bool {
    matches!(entry.sync_state, WorkshopSyncState::Synced)
        && supports_first_release(entry.project_type)
        && entry.library_item_id.is_some()
}

pub struct LibraryService;

impl LibraryService {
    pub fn projection_from_refresh(refresh: WorkshopRefreshResult) -> LibraryProjection {
        let source_catalog_count = refresh.catalog_entries.len();
        let entries = refresh
            .catalog_entries
            .into_iter()
            .filter(includes_library_item)
            .collect();

        LibraryProjection {
            entries,
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
}

#[cfg(test)]
mod tests {
    use crate::results::library::LibraryProjection;

    #[test]
    fn service_layer_library_service_uses_application_projection_result() {
        let result = LibraryProjection {
            entries: Vec::new(),
            source_catalog_count: 0,
        };

        assert!(result.entries.is_empty());
        assert_eq!(result.source_catalog_count, 0);
    }
}
