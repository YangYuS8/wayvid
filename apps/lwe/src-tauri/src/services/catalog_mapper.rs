use crate::models::{
    CompatibilityBadge, ItemType, LibraryItemDetail, LibraryItemSummary, LibrarySource,
    WorkshopItemDetail, WorkshopItemSummary, WorkshopSyncStatus,
};
use crate::policies::shared::compatibility_policy::{
    compatibility_decision, CompatibilityLevel, CompatibilityReason,
};
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::policies::shared::support_policy::supports_first_release;
use wayvid_library::{WeProject, WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

pub(super) fn item_type_from_project_type(project_type: WorkshopProjectType) -> ItemType {
    match project_type {
        WorkshopProjectType::Video => ItemType::Video,
        WorkshopProjectType::Scene => ItemType::Scene,
        WorkshopProjectType::Web => ItemType::Web,
        WorkshopProjectType::Other => ItemType::Other,
    }
}

pub(super) fn cover_path(entry: &WorkshopCatalogEntry) -> Option<String> {
    let bundled_cover_path = entry
        .cover_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());

    match cover_art_source(bundled_cover_path) {
        CoverArtSource::Bundled(path) => Some(path),
        CoverArtSource::Placeholder => None,
    }
}

pub(super) fn includes_library_item(entry: &WorkshopCatalogEntry) -> bool {
    matches!(entry.sync_state, WorkshopSyncState::Synced)
        && supports_first_release(entry.project_type)
        && entry.library_item_id.is_some()
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

fn compatibility_note(entry: &WorkshopCatalogEntry) -> Option<String> {
    match compatibility_decision(entry).reason {
        CompatibilityReason::ReadyForLibrary => {
            Some("This item is synchronized locally and available in the Library page.".to_string())
        }
        CompatibilityReason::MissingProjectMetadata => Some(
            "The local Workshop folder is missing valid project metadata, so LWE cannot classify or import this item yet."
                .to_string(),
        ),
        CompatibilityReason::MissingPrimaryAsset => Some(
            "The project metadata was found, but the primary local asset is missing, so it cannot be projected into Library yet."
                .to_string(),
        ),
        CompatibilityReason::UnsupportedWebItem => Some(
            "Web Workshop items are visible here, but the first release only supports video and scene imports."
                .to_string(),
        ),
        CompatibilityReason::UnsupportedProjectType => Some(
            "This Workshop item uses a project type that the first release does not import yet."
                .to_string(),
        ),
    }
}

pub(super) fn workshop_summary_from_entry(entry: WorkshopCatalogEntry) -> WorkshopItemSummary {
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = cover_path(&entry);
    let sync_status = sync_status(&entry);
    let compatibility_badge = compatibility_badge(&entry);

    WorkshopItemSummary {
        id: entry.workshop_id.to_string(),
        title: entry.title,
        item_type,
        cover_path,
        sync_status,
        compatibility_badge,
    }
}

pub(super) fn workshop_detail_from_entry(entry: WorkshopCatalogEntry) -> WorkshopItemDetail {
    let project = WeProject::load(&entry.project_dir).ok();
    let description = project
        .as_ref()
        .and_then(|project| project.description.clone());
    let tags = project.map(|project| project.tags).unwrap_or_default();
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = cover_path(&entry);
    let sync_status = sync_status(&entry);
    let compatibility_badge = compatibility_badge(&entry);
    let compatibility_note = compatibility_note(&entry);

    WorkshopItemDetail {
        id: entry.workshop_id.to_string(),
        title: entry.title,
        item_type,
        cover_path,
        sync_status,
        compatibility_badge,
        compatibility_note,
        tags,
        description,
    }
}

pub(super) fn library_summary_from_entry(entry: WorkshopCatalogEntry) -> LibraryItemSummary {
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = cover_path(&entry);

    LibraryItemSummary {
        id: entry.library_item_id.unwrap_or_default(),
        title: entry.title,
        item_type,
        cover_path,
        source: LibrarySource::Workshop,
        favorite: false,
    }
}

pub(super) fn library_detail_from_entry(entry: WorkshopCatalogEntry) -> LibraryItemDetail {
    let item_type = item_type_from_project_type(entry.project_type);
    let cover_path = cover_path(&entry);
    let project = WeProject::load(&entry.project_dir).ok();
    let description = project
        .as_ref()
        .and_then(|project| project.description.clone());
    let tags = project.map(|project| project.tags).unwrap_or_default();

    LibraryItemDetail {
        id: entry.library_item_id.unwrap_or_default(),
        title: entry.title,
        item_type,
        cover_path,
        source: LibrarySource::Workshop,
        description,
        tags,
    }
}
