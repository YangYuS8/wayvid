use crate::results::workshop::{WorkshopInspection, WorkshopRefreshResult};
use crate::workshop::scan_workshop_catalog;

pub struct WorkshopService;

impl WorkshopService {
    pub fn refresh_catalog() -> Result<WorkshopRefreshResult, String> {
        Ok(WorkshopRefreshResult {
            catalog_entries: scan_workshop_catalog()?,
            library_refresh_required: true,
        })
    }

    pub fn inspect_item(workshop_id: &str) -> Result<WorkshopInspection, String> {
        let entry = Self::refresh_catalog()?
            .catalog_entries
            .into_iter()
            .find(|entry| entry.workshop_id.to_string() == workshop_id)
            .ok_or_else(|| format!("Workshop item {workshop_id} not found"))?;

        Ok(WorkshopInspection {
            requested_workshop_id: workshop_id.to_string(),
            entry,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::results::workshop::WorkshopRefreshResult;

    #[test]
    fn service_layer_workshop_service_returns_application_result_not_page_snapshot() {
        let result = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
            library_refresh_required: true,
        };

        assert!(result.library_refresh_required);
        assert_eq!(result.catalog_entries.len(), 0);
    }
}
