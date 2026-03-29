use crate::models::LibraryPageSnapshot;
use crate::results::library::LibraryProjection;

pub fn assemble_library_page(result: LibraryProjection) -> LibraryPageSnapshot {
    LibraryPageSnapshot {
        items: result.projected_items,
        selected_item_id: None,
        stale: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::library::LibraryProjection;

    #[test]
    fn assembler_turns_library_projection_into_page_snapshot() {
        let snapshot = assemble_library_page(LibraryProjection {
            projected_items: Vec::new(),
            source_catalog_count: 0,
        });

        assert!(snapshot.items.is_empty());
    }
}
