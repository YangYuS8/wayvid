use crate::policies::shared::compatibility_policy::{
    CompatibilityLevel, compatibility_decision, scene_compatibility_decision,
};
use crate::results::workshop::{AssessedWorkshopCatalogEntry, WorkshopProjectMetadata};
use crate::services::scene_service::SceneService;
use lwe_library::{
    WallpaperItem, WeProject, WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState,
};
use lwe_scene::SceneAssets;

pub struct CompatibilityService;

impl CompatibilityService {
    fn project_metadata(entry: &WorkshopCatalogEntry) -> WorkshopProjectMetadata {
        let mut metadata = WeProject::load(&entry.project_dir)
            .map(|project| WorkshopProjectMetadata {
                description: project.description,
                tags: project.tags,
                inferred_age_rating: None,
                scene_assessment: None,
            })
            .unwrap_or_default();
        if entry.project_type == WorkshopProjectType::Scene
            && entry.sync_state != WorkshopSyncState::MissingProjectFile
        {
            metadata.scene_assessment = Some(SceneService::assess_project(&entry.project_dir));
        }
        metadata
    }

    pub fn assess_catalog_entry(mut entry: WorkshopCatalogEntry) -> AssessedWorkshopCatalogEntry {
        if entry.project_type == WorkshopProjectType::Scene
            && entry.sync_state != WorkshopSyncState::MissingProjectFile
            && let Ok(project) = WeProject::load(&entry.project_dir)
            && project.is_scene()
        {
            // The retained scanner expects a loose primary file. Preserve its
            // logical source identity when the primary is packaged instead.
            if entry.library_item_id.is_none() {
                let source = project
                    .main_file(&entry.project_dir)
                    .unwrap_or_else(|| entry.project_dir.clone());
                entry.library_item_id = Some(WallpaperItem::generate_id(&source));
            }
            if entry.sync_state == WorkshopSyncState::MissingPrimaryAsset
                && let Some(primary) = project.file.as_deref()
                && let Ok(assets) = SceneAssets::open(&entry.project_dir, None)
                && assets.exists(primary)
            {
                entry.sync_state = WorkshopSyncState::Synced;
            }
        }
        let project_metadata = Self::project_metadata(&entry);
        let compatibility = project_metadata
            .scene_assessment
            .as_ref()
            .map(scene_compatibility_decision)
            .unwrap_or_else(|| compatibility_decision(&entry));

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
        entry.entry.library_item_id.is_some()
            && (entry.compatibility.level == CompatibilityLevel::FullySupported
                || (entry.entry.project_type == WorkshopProjectType::Scene
                    && entry.entry.sync_state != WorkshopSyncState::MissingProjectFile))
    }

    pub fn supports_runtime(entry: &AssessedWorkshopCatalogEntry) -> bool {
        match entry.entry.project_type {
            WorkshopProjectType::Video => {
                entry.compatibility.level == CompatibilityLevel::FullySupported
            }
            WorkshopProjectType::Scene => entry
                .project_metadata
                .scene_assessment
                .as_ref()
                .is_some_and(|assessment| assessment.runnable),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policies::shared::compatibility_policy::{
        CompatibilityNextStep, CompatibilityReason,
    };
    use lwe_library::{SteamLibrary, WorkshopProjectType, WorkshopScanner, WorkshopSyncState};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn packaged_scene_catalog(package: Option<&[u8]>) -> (TempDir, WorkshopCatalogEntry) {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path().join("steamapps/workshop/content/431960/48");
        fs::create_dir_all(&project_dir).unwrap();
        // A local assets directory keeps this fixture independent of the user's Steam setup.
        fs::create_dir_all(temp.path().join("steamapps/common/wallpaper_engine/assets")).unwrap();
        fs::write(
            project_dir.join("project.json"),
            r#"{"type":"scene","title":"Packaged Scene","file":"scene.json"}"#,
        )
        .unwrap();
        if let Some(package) = package {
            fs::write(project_dir.join("scene.pkg"), package).unwrap();
        }
        let mut scanner = WorkshopScanner::new(SteamLibrary {
            root: temp.path().to_path_buf(),
            libraries: Vec::new(),
        });
        let entry = scanner.scan_catalog().unwrap().pop().unwrap();
        (temp, entry)
    }

    fn package_with_scene(scene: &[u8]) -> Vec<u8> {
        let mut package = Vec::new();
        package.extend_from_slice(&8_u32.to_le_bytes());
        package.extend_from_slice(b"PKGV0023");
        package.extend_from_slice(&1_u32.to_le_bytes());
        package.extend_from_slice(&10_u32.to_le_bytes());
        package.extend_from_slice(b"scene.json");
        package.extend_from_slice(&0_u32.to_le_bytes());
        package.extend_from_slice(&(scene.len() as u32).to_le_bytes());
        package.extend_from_slice(scene);
        package
    }

    fn synced_video_entry() -> WorkshopCatalogEntry {
        WorkshopCatalogEntry {
            workshop_id: 42,
            title: "Forest Video".to_string(),
            project_type: WorkshopProjectType::Video,
            project_dir: PathBuf::from("/tmp/42"),
            cover_path: None,
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("video-42".to_string()),
        }
    }

    fn catalog_entry(
        workshop_id: u64,
        project_type: WorkshopProjectType,
        sync_state: WorkshopSyncState,
        library_item_id: Option<&str>,
    ) -> WorkshopCatalogEntry {
        WorkshopCatalogEntry {
            workshop_id,
            title: format!("Item {workshop_id}"),
            project_type,
            project_dir: PathBuf::from(format!("/tmp/{workshop_id}")),
            cover_path: None,
            sync_state,
            supported_first_release: matches!(project_type, WorkshopProjectType::Video)
                && matches!(sync_state, WorkshopSyncState::Synced),
            library_item_id: library_item_id.map(str::to_string),
        }
    }

    #[test]
    fn compatibility_service_assesses_catalog_entries_once_for_service_consumers() {
        let assessed = CompatibilityService::assess_catalog_entry(synced_video_entry());

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
        let assessed = CompatibilityService::assess_catalog_entry(synced_video_entry());

        assert!(CompatibilityService::supports_library_projection(&assessed));
    }

    #[test]
    fn compatibility_service_includes_recognized_scene_in_library_projection() {
        let assessed = CompatibilityService::assess_catalog_entry(WorkshopCatalogEntry {
            workshop_id: 43,
            title: "Forest Scene".to_string(),
            project_type: WorkshopProjectType::Scene,
            project_dir: PathBuf::from("/tmp/43"),
            cover_path: None,
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: false,
            library_item_id: Some("scene-43".to_string()),
        });

        assert_eq!(
            assessed.compatibility.reason,
            CompatibilityReason::SceneAssessmentFailed
        );
        assert!(CompatibilityService::supports_library_projection(&assessed));
    }

    #[test]
    fn packaged_primary_enters_library_but_unknown_scene_feature_cannot_run() {
        let package = package_with_scene(
            br#"{"general":{"orthogonalprojection":{"width":4,"height":4}},"objects":[{"future":"unknown"}]}"#,
        );
        let (temp, scanned) = packaged_scene_catalog(Some(&package));
        assert_eq!(scanned.sync_state, WorkshopSyncState::MissingPrimaryAsset);
        assert!(scanned.library_item_id.is_none());

        let assessed = CompatibilityService::assess_catalog_entry(scanned);

        assert_eq!(assessed.entry.sync_state, WorkshopSyncState::Synced);
        assert_eq!(
            assessed.entry.library_item_id,
            Some(WallpaperItem::generate_id(
                &assessed.entry.project_dir.join("scene.json")
            ))
        );
        assert!(CompatibilityService::supports_library_projection(&assessed));
        assert!(!CompatibilityService::supports_runtime(&assessed));
        assert_eq!(
            assessed.compatibility.reason,
            CompatibilityReason::SceneUnsupportedFeatures
        );

        let scene = SceneAssets::open(&assessed.entry.project_dir, None)
            .unwrap()
            .read("scene.json", 4 * 1024 * 1024)
            .unwrap();
        fs::write(assessed.entry.project_dir.join("scene.json"), scene).unwrap();
        let mut scanner = WorkshopScanner::new(SteamLibrary {
            root: temp.path().to_path_buf(),
            libraries: Vec::new(),
        });
        let unpacked = scanner.scan_catalog().unwrap().pop().unwrap();
        assert_eq!(unpacked.library_item_id, assessed.entry.library_item_id);
        assert_eq!(
            assessed.project_metadata.scene_assessment.unwrap().issues[0].code,
            "unsupported_scene"
        );
    }

    #[test]
    fn absent_or_corrupt_package_preserves_missing_primary_and_blocks_runtime() {
        for package in [None, Some(b"broken package".as_slice())] {
            let (_temp, scanned) = packaged_scene_catalog(package);
            let assessed = CompatibilityService::assess_catalog_entry(scanned);

            assert_eq!(
                assessed.entry.sync_state,
                WorkshopSyncState::MissingPrimaryAsset
            );
            assert!(CompatibilityService::supports_library_projection(&assessed));
            assert!(!CompatibilityService::supports_runtime(&assessed));
            assert_eq!(
                assessed.compatibility.reason,
                CompatibilityReason::SceneAssessmentFailed
            );
        }
    }

    #[test]
    fn compatibility_service_excludes_degraded_video_without_runtime_asset_from_library_projection()
    {
        let assessed = AssessedWorkshopCatalogEntry {
            entry: WorkshopCatalogEntry {
                workshop_id: 44,
                title: "Broken Video".to_string(),
                project_type: WorkshopProjectType::Video,
                project_dir: PathBuf::from("/tmp/44"),
                cover_path: None,
                sync_state: WorkshopSyncState::MissingPrimaryAsset,
                supported_first_release: false,
                library_item_id: Some("video-44".to_string()),
            },
            compatibility: crate::policies::shared::compatibility_policy::CompatibilityDecision {
                level: CompatibilityLevel::PartiallySupported,
                reason: CompatibilityReason::MissingPrimaryAsset,
                next_step: CompatibilityNextStep::ResyncWorkshopItem,
            },
            project_metadata: WorkshopProjectMetadata::default(),
        };

        assert!(!CompatibilityService::supports_library_projection(
            &assessed
        ));
    }

    #[test]
    fn compatibility_service_excludes_web_items_from_library_projection() {
        let assessed = CompatibilityService::assess_catalog_entry(catalog_entry(
            45,
            WorkshopProjectType::Web,
            WorkshopSyncState::UnsupportedType,
            Some("web-45"),
        ));

        assert_eq!(
            assessed.compatibility.reason,
            CompatibilityReason::UnsupportedWebItem
        );
        assert!(!CompatibilityService::supports_library_projection(
            &assessed
        ));
    }

    #[test]
    fn compatibility_service_excludes_missing_project_metadata_from_library_projection() {
        let assessed = CompatibilityService::assess_catalog_entry(catalog_entry(
            46,
            WorkshopProjectType::Video,
            WorkshopSyncState::MissingProjectFile,
            Some("video-46"),
        ));

        assert_eq!(
            assessed.compatibility.reason,
            CompatibilityReason::MissingProjectMetadata
        );
        assert!(!CompatibilityService::supports_library_projection(
            &assessed
        ));
    }

    #[test]
    fn compatibility_service_preserves_scene_runtime_limitation_for_missing_asset() {
        let assessed = CompatibilityService::assess_catalog_entry(catalog_entry(
            47,
            WorkshopProjectType::Scene,
            WorkshopSyncState::MissingPrimaryAsset,
            Some("scene-47"),
        ));

        assert_eq!(
            assessed.compatibility.reason,
            CompatibilityReason::SceneAssessmentFailed
        );
        assert!(CompatibilityService::supports_library_projection(&assessed));
    }
}
