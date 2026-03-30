use crate::assembly::compatibility::compatibility_explanation;
use crate::models::{ItemType, WorkshopItemDetail, WorkshopSyncStatus};
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::results::workshop::WorkshopInspection;
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
    let compatibility = compatibility_explanation(&entry.compatibility);

    WorkshopItemDetail {
        id,
        title,
        item_type,
        cover_path,
        sync_status,
        compatibility,
        tags,
        description,
    }
}
