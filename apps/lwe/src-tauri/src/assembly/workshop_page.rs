use crate::models::{
    CompatibilityBadge, ItemType, WorkshopItemSummary, WorkshopPageSnapshot, WorkshopSyncStatus,
};
use crate::policies::shared::compatibility_policy::{compatibility_decision, CompatibilityLevel};
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::results::workshop::WorkshopRefreshResult;
use wayvid_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

fn item_type_from_project_type(project_type: WorkshopProjectType) -> ItemType {
    match project_type {
        WorkshopProjectType::Video => ItemType::Video,
        WorkshopProjectType::Scene => ItemType::Scene,
        WorkshopProjectType::Web => ItemType::Web,
        WorkshopProjectType::Other => ItemType::Other,
    }
}

fn cover_path(entry: &WorkshopCatalogEntry) -> Option<String> {
    let bundled_cover_path = entry
        .cover_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());

    match cover_art_source(bundled_cover_path) {
        CoverArtSource::Bundled(path) => Some(path),
        CoverArtSource::Placeholder => None,
    }
}

fn sync_status(entry: &WorkshopCatalogEntry) -> WorkshopSyncStatus {
    match entry.sync_state {
        WorkshopSyncState::Synced => WorkshopSyncStatus::Synced,
        WorkshopSyncState::MissingProjectFile => WorkshopSyncStatus::MissingProject,
        WorkshopSyncState::MissingPrimaryAsset => WorkshopSyncStatus::MissingAsset,
        WorkshopSyncState::UnsupportedType => WorkshopSyncStatus::UnsupportedType,
    }
}

fn compatibility_badge(entry: &WorkshopCatalogEntry) -> CompatibilityBadge {
    match compatibility_decision(entry).level {
        CompatibilityLevel::FullySupported => CompatibilityBadge::FullySupported,
        CompatibilityLevel::PartiallySupported => CompatibilityBadge::PartiallySupported,
        CompatibilityLevel::Unsupported => CompatibilityBadge::Unsupported,
    }
}

fn workshop_summary_from_entry(entry: WorkshopCatalogEntry) -> WorkshopItemSummary {
    let workshop_id = entry.workshop_id.to_string();
    let title = entry.title.clone();
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = cover_path(&entry);
    let sync_status = sync_status(&entry);
    let compatibility_badge = compatibility_badge(&entry);

    WorkshopItemSummary {
        id: workshop_id,
        title,
        item_type,
        cover_path,
        sync_status,
        compatibility_badge,
    }
}

pub fn assemble_workshop_page(result: &WorkshopRefreshResult) -> WorkshopPageSnapshot {
    WorkshopPageSnapshot {
        items: result
            .catalog_entries
            .clone()
            .into_iter()
            .map(workshop_summary_from_entry)
            .collect(),
        selected_item_id: None,
        stale: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::workshop::WorkshopRefreshResult;

    #[test]
    fn assembler_turns_app_result_into_page_snapshot() {
        let result = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
            library_refresh_required: true,
        };

        let snapshot = assemble_workshop_page(&result);
        assert!(snapshot.items.is_empty());
    }
}
