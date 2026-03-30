use crate::results::desktop::DesktopPageResult;
use crate::results::library::LibraryProjection;
use crate::results::workshop::AssessedWorkshopCatalogEntry;
use crate::results::workshop::WorkshopRefreshResult;
use crate::services::compatibility_service::CompatibilityService;
use crate::services::workshop_service::WorkshopService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryDesktopStatus {
    pub monitor_discovery_issue: Option<String>,
    pub desktop_assignment_issue: Option<String>,
    pub desktop_assignments_available: bool,
    pub stale: bool,
}

fn includes_library_item(entry: &AssessedWorkshopCatalogEntry) -> bool {
    CompatibilityService::supports_library_projection(entry)
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

    pub fn desktop_status(desktop: &DesktopPageResult) -> LibraryDesktopStatus {
        LibraryDesktopStatus {
            monitor_discovery_issue: desktop.monitor_discovery_issue.clone(),
            desktop_assignment_issue: desktop.persistence_issue.clone(),
            desktop_assignments_available: desktop.assignments_available,
            stale: desktop.stale,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::results::desktop::DesktopPageResult;
    use crate::results::library::LibraryProjection;

    use super::LibraryService;

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
    fn service_layer_library_service_preserves_unavailable_desktop_assignment_state() {
        let status = LibraryService::desktop_status(&DesktopPageResult {
            monitors: Vec::new(),
            assignments: BTreeMap::new(),
            monitors_available: false,
            monitor_discovery_issue: Some("Monitor discovery is not available yet".to_string()),
            persistence_issue: Some("Desktop persistence is not available yet".to_string()),
            assignments_available: false,
            stale: true,
        });

        assert_eq!(
            status.monitor_discovery_issue.as_deref(),
            Some("Monitor discovery is not available yet")
        );
        assert_eq!(
            status.desktop_assignment_issue.as_deref(),
            Some("Desktop persistence is not available yet")
        );
        assert!(!status.desktop_assignments_available);
        assert!(status.stale);
    }
}
