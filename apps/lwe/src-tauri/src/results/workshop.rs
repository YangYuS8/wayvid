use wayvid_library::WorkshopCatalogEntry;

#[derive(Debug, Clone)]
pub struct WorkshopRefreshResult {
    pub catalog_entries: Vec<WorkshopCatalogEntry>,
}

#[derive(Debug, Clone)]
pub struct WorkshopInspection {
    pub entry: WorkshopCatalogEntry,
}
