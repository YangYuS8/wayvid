use crate::policies::shared::support_policy::supports_first_release;
use lwe_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityLevel {
    FullySupported,
    PartiallySupported,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityReason {
    ReadyForLibrary,
    MissingProjectMetadata,
    MissingPrimaryAsset,
    UnsupportedWebItem,
    UnsupportedProjectType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatibilityDecision {
    pub level: CompatibilityLevel,
    pub reason: CompatibilityReason,
}

pub fn compatibility_decision(entry: &WorkshopCatalogEntry) -> CompatibilityDecision {
    let supported_first_release = supports_first_release(entry.project_type);

    match (
        supported_first_release,
        entry.sync_state,
        entry.project_type,
    ) {
        (true, WorkshopSyncState::Synced, _) => CompatibilityDecision {
            level: CompatibilityLevel::FullySupported,
            reason: CompatibilityReason::ReadyForLibrary,
        },
        (_, WorkshopSyncState::MissingProjectFile, _) => CompatibilityDecision {
            level: CompatibilityLevel::Unsupported,
            reason: CompatibilityReason::MissingProjectMetadata,
        },
        (
            _,
            WorkshopSyncState::MissingPrimaryAsset,
            WorkshopProjectType::Video | WorkshopProjectType::Scene,
        ) => CompatibilityDecision {
            level: CompatibilityLevel::PartiallySupported,
            reason: CompatibilityReason::MissingPrimaryAsset,
        },
        (_, WorkshopSyncState::UnsupportedType, WorkshopProjectType::Web) => {
            CompatibilityDecision {
                level: CompatibilityLevel::Unsupported,
                reason: CompatibilityReason::UnsupportedWebItem,
            }
        }
        _ => CompatibilityDecision {
            level: CompatibilityLevel::Unsupported,
            reason: CompatibilityReason::UnsupportedProjectType,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn unsupported_web_entry() -> WorkshopCatalogEntry {
        WorkshopCatalogEntry {
            workshop_id: 55,
            title: "Web".to_string(),
            project_type: WorkshopProjectType::Web,
            project_dir: PathBuf::from("/tmp/55"),
            cover_path: None,
            sync_state: WorkshopSyncState::UnsupportedType,
            supported_first_release: false,
            library_item_id: None,
        }
    }

    #[test]
    fn shared_policy_compatibility_decision_returns_reason_codes() {
        let decision = compatibility_decision(&unsupported_web_entry());

        assert_eq!(decision.level, CompatibilityLevel::Unsupported);
        assert_eq!(decision.reason, CompatibilityReason::UnsupportedWebItem);
    }

    #[test]
    fn shared_policy_support_matrix_is_authoritative_for_synced_items() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 77,
            title: "Unexpected Type".to_string(),
            project_type: WorkshopProjectType::Other,
            project_dir: PathBuf::from("/tmp/77"),
            cover_path: None,
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: None,
        };

        let decision = compatibility_decision(&entry);

        assert_eq!(decision.level, CompatibilityLevel::Unsupported);
        assert_eq!(decision.reason, CompatibilityReason::UnsupportedProjectType);
    }
}
