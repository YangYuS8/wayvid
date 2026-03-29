use lwe_library::{WorkshopCatalogEntry, WorkshopSyncState};

#[derive(Debug, Clone)]
pub struct WorkshopRefreshResult {
    pub catalog_entries: Vec<WorkshopCatalogEntry>,
    pub library_refresh_required: bool,
}

impl WorkshopRefreshResult {
    pub fn synced_entry_count(&self) -> usize {
        self.catalog_entries
            .iter()
            .filter(|entry| entry.sync_state == WorkshopSyncState::Synced)
            .count()
    }
}

#[derive(Debug, Clone)]
pub struct WorkshopInspection {
    pub requested_workshop_id: String,
    pub entry: WorkshopCatalogEntry,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_policy_workshop_refresh_result_tracks_library_invalidation() {
        let result = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
            library_refresh_required: true,
        };

        assert!(result.library_refresh_required);
    }
}
