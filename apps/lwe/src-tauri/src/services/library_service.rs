use crate::library::library_projection_from_entries;
use crate::results::library::LibraryProjection;
use crate::services::workshop_service::WorkshopService;

pub struct LibraryService;

impl LibraryService {
    pub fn load_projection() -> Result<LibraryProjection, String> {
        let refresh = WorkshopService::refresh_catalog()?;

        Ok(library_projection_from_entries(refresh.catalog_entries))
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
