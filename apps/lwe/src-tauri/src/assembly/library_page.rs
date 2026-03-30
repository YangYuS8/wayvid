use crate::assembly::compatibility::compatibility_summary;
use crate::models::LibraryPageSnapshot;
use crate::models::{ItemType, LibraryItemSummary, LibrarySource};
use crate::policies::shared::cover_policy::{cover_art_source, CoverArtSource};
use crate::results::library::LibraryProjection;
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

fn assemble_library_summary(entry: AssessedWorkshopCatalogEntry) -> LibraryItemSummary {
    LibraryItemSummary {
        id: entry.entry.library_item_id.clone().unwrap_or_default(),
        title: entry.entry.title.clone(),
        item_type: item_type_from_project_type(entry.entry.project_type),
        cover_path: cover_path(&entry.entry),
        source: LibrarySource::Workshop,
        compatibility: compatibility_summary(&entry.compatibility),
        favorite: false,
    }
}

pub fn assemble_library_page(result: LibraryProjection) -> LibraryPageSnapshot {
    LibraryPageSnapshot {
        items: result
            .entries
            .into_iter()
            .map(assemble_library_summary)
            .collect(),
        selected_item_id: None,
        stale: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policies::shared::compatibility_policy::{
        CompatibilityDecision, CompatibilityLevel, CompatibilityNextStep, CompatibilityReason,
    };
    use crate::results::library::LibraryProjection;
    use crate::results::workshop::AssessedWorkshopCatalogEntry;
    use lwe_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

    fn assessed_entry() -> AssessedWorkshopCatalogEntry {
        AssessedWorkshopCatalogEntry {
            entry: WorkshopCatalogEntry {
                workshop_id: 7,
                title: "Forest Scene".to_string(),
                project_type: WorkshopProjectType::Scene,
                project_dir: std::path::PathBuf::from("/tmp/7"),
                cover_path: None,
                sync_state: WorkshopSyncState::Synced,
                supported_first_release: true,
                library_item_id: Some("scene-7".to_string()),
            },
            compatibility: CompatibilityDecision {
                level: CompatibilityLevel::FullySupported,
                reason: CompatibilityReason::ReadyForLibrary,
                next_step: CompatibilityNextStep::None,
            },
        }
    }

    #[test]
    fn assembler_turns_library_projection_entries_into_page_snapshot() {
        let snapshot = assemble_library_page(LibraryProjection {
            entries: vec![assessed_entry()],
            source_catalog_count: 1,
        });

        assert_eq!(snapshot.items.len(), 1);
        assert_eq!(snapshot.items[0].id, "scene-7");
        assert_eq!(snapshot.items[0].title, "Forest Scene");
    }
}
