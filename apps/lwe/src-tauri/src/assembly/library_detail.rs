use crate::assembly::compatibility::compatibility_explanation;
use crate::models::{ItemType, LibraryItemDetail, LibrarySource};
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::results::workshop::AssessedWorkshopCatalogEntry;
use lwe_library::{WorkshopCatalogEntry, WorkshopProjectType};

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

pub fn assemble_library_detail(entry: AssessedWorkshopCatalogEntry) -> LibraryItemDetail {
    let id = entry.entry.library_item_id.clone().unwrap_or_default();
    let title = entry.entry.title.clone();
    let item_type = item_type_from_project_type(entry.entry.project_type);
    let cover_path = cover_path(&entry.entry);
    let description = entry.project_metadata.description.clone();
    let tags = entry.project_metadata.tags.clone();

    LibraryItemDetail {
        id,
        title,
        item_type,
        cover_path,
        source: LibrarySource::Workshop,
        compatibility: compatibility_explanation(&entry.compatibility),
        description,
        tags,
    }
}
