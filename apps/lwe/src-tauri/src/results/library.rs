use crate::models::LibraryItemSummary;

#[derive(Debug, Clone)]
pub struct LibraryProjection {
    pub projected_items: Vec<LibraryItemSummary>,
    pub source_catalog_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_policy_library_projection_records_source_catalog_size() {
        let projection = LibraryProjection {
            projected_items: Vec::new(),
            source_catalog_count: 3,
        };

        assert_eq!(projection.source_catalog_count, 3);
        assert!(projection.projected_items.is_empty());
    }
}
