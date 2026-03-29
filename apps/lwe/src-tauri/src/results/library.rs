use lwe_library::WorkshopCatalogEntry;

#[derive(Debug, Clone)]
pub struct LibraryProjection {
    pub entries: Vec<WorkshopCatalogEntry>,
    pub source_catalog_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_policy_library_projection_records_source_catalog_size() {
        let projection = LibraryProjection {
            entries: Vec::new(),
            source_catalog_count: 3,
        };

        assert_eq!(projection.source_catalog_count, 3);
        assert!(projection.entries.is_empty());
    }
}
