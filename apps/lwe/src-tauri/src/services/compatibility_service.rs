use crate::policies::shared::compatibility_policy::{
    compatibility_decision, CompatibilityLevel, CompatibilityReason,
};
use crate::results::compatibility::CompatibilityAssessment;
use crate::results::workshop::{AssessedWorkshopCatalogEntry, WorkshopProjectMetadata};
use lwe_library::{WeProject, WorkshopCatalogEntry};

pub struct CompatibilityService;

impl CompatibilityService {
    fn project_metadata(entry: &WorkshopCatalogEntry) -> WorkshopProjectMetadata {
        WeProject::load(&entry.project_dir)
            .map(|project| WorkshopProjectMetadata {
                description: project.description,
                tags: project.tags,
            })
            .unwrap_or_default()
    }

    pub fn assess_catalog_entry(entry: WorkshopCatalogEntry) -> AssessedWorkshopCatalogEntry {
        let compatibility = compatibility_decision(&entry);
        let project_metadata = Self::project_metadata(&entry);

        AssessedWorkshopCatalogEntry {
            entry,
            compatibility,
            project_metadata,
        }
    }

    pub fn assess_catalog_entries(
        entries: Vec<WorkshopCatalogEntry>,
    ) -> Vec<AssessedWorkshopCatalogEntry> {
        entries
            .into_iter()
            .map(Self::assess_catalog_entry)
            .collect()
    }

    pub fn supports_library_projection(entry: &AssessedWorkshopCatalogEntry) -> bool {
        entry.compatibility.level == CompatibilityLevel::FullySupported
            && entry.entry.library_item_id.is_some()
    }

    pub fn compatibility_note(assessment: CompatibilityAssessment) -> Option<String> {
        match assessment.reason {
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
                "This Workshop item uses a project type that the first release does not import yet.".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lwe_library::{WorkshopProjectType, WorkshopSyncState};
    use std::path::PathBuf;

    fn synced_scene_entry() -> WorkshopCatalogEntry {
        WorkshopCatalogEntry {
            workshop_id: 42,
            title: "Forest Scene".to_string(),
            project_type: WorkshopProjectType::Scene,
            project_dir: PathBuf::from("/tmp/42"),
            cover_path: None,
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("scene-42".to_string()),
        }
    }

    #[test]
    fn compatibility_service_assesses_catalog_entries_once_for_service_consumers() {
        let assessed = CompatibilityService::assess_catalog_entry(synced_scene_entry());

        assert_eq!(
            assessed.compatibility.level,
            CompatibilityLevel::FullySupported
        );
        assert_eq!(
            assessed.compatibility.reason,
            CompatibilityReason::ReadyForLibrary
        );
    }

    #[test]
    fn compatibility_service_uses_assessment_for_library_projection_gate() {
        let assessed = CompatibilityService::assess_catalog_entry(synced_scene_entry());

        assert!(CompatibilityService::supports_library_projection(&assessed));
    }
}
