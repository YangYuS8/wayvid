use crate::models::LibraryItemSummary;

#[derive(Debug, Clone)]
pub struct LibraryProjection {
    pub items: Vec<LibraryItemSummary>,
}
