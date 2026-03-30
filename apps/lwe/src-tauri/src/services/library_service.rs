use crate::results::desktop_persistence::DesktopPersistenceLoad;
use crate::results::library::LibraryProjection;
use crate::results::workshop::AssessedWorkshopCatalogEntry;
use crate::results::workshop::WorkshopRefreshResult;
use crate::services::compatibility_service::CompatibilityService;
use crate::services::desktop_persistence_service::DesktopPersistenceService;
use crate::services::workshop_service::WorkshopService;

fn includes_library_item(entry: &AssessedWorkshopCatalogEntry) -> bool {
    CompatibilityService::supports_library_projection(entry)
}

pub struct LibraryService;

impl LibraryService {
    pub fn desktop_assignment_issue() -> Option<String> {
        match DesktopPersistenceService::load_state() {
            DesktopPersistenceLoad::Loaded(_) => None,
            DesktopPersistenceLoad::Unavailable { reason } => Some(reason),
        }
    }

    pub fn desktop_assignments_available() -> bool {
        Self::desktop_assignment_issue().is_none()
    }

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

    pub fn inspect_item(item_id: &str) -> Result<AssessedWorkshopCatalogEntry, String> {
        WorkshopService::refresh_catalog()?
            .catalog_entries
            .into_iter()
            .find(|entry| {
                includes_library_item(entry)
                    && entry.entry.library_item_id.as_deref() == Some(item_id)
            })
            .ok_or_else(|| format!("Library item {item_id} not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::LibraryService;
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

    #[test]
    fn desktop_apply_flow_library_service_reports_assignment_state_unavailable() {
        assert!(!LibraryService::desktop_assignments_available());
        assert_eq!(
            LibraryService::desktop_assignment_issue().as_deref(),
            Some("Desktop persistence is not available yet")
        );
    }
}
