use crate::models::{CompatibilityBadge, ItemType, WorkshopItemDetail, WorkshopSyncStatus};
use crate::policies::shared::compatibility_policy::CompatibilityLevel;
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::results::workshop::{AssessedWorkshopCatalogEntry, WorkshopInspection};
use crate::services::compatibility_service::CompatibilityService;
use lwe_library::{WeProject, WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

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

fn compatibility_badge(entry: &AssessedWorkshopCatalogEntry) -> CompatibilityBadge {
    match entry.compatibility.level {
        CompatibilityLevel::FullySupported => CompatibilityBadge::FullySupported,
        CompatibilityLevel::PartiallySupported => CompatibilityBadge::PartiallySupported,
        CompatibilityLevel::Unsupported => CompatibilityBadge::Unsupported,
    }
}

pub fn assemble_workshop_detail(result: WorkshopInspection) -> WorkshopItemDetail {
    let entry = result.entry;
    let project = WeProject::load(&entry.entry.project_dir).ok();
    let description = project
        .as_ref()
        .and_then(|project| project.description.clone());
    let tags = project.map(|project| project.tags).unwrap_or_default();
    let id = entry.entry.workshop_id.to_string();
    let title = entry.entry.title.clone();
    let item_type = item_type_from_project_type(entry.entry.project_type);
    let cover_path = cover_path(&entry.entry);
    let sync_status = sync_status(&entry.entry);
    let compatibility_badge = compatibility_badge(&entry);
    let compatibility_note = CompatibilityService::compatibility_note(entry.compatibility);

    WorkshopItemDetail {
        id,
        title,
        item_type,
        cover_path,
        sync_status,
        compatibility_badge,
        compatibility_note,
        tags,
        description,
    }
}
