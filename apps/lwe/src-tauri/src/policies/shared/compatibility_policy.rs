use crate::models::CompatibilityBadge;
use wayvid_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

pub fn compatibility_badge(entry: &WorkshopCatalogEntry) -> CompatibilityBadge {
    if entry.supported_first_release {
        CompatibilityBadge::FullySupported
    } else if matches!(entry.sync_state, WorkshopSyncState::MissingPrimaryAsset) {
        CompatibilityBadge::PartiallySupported
    } else {
        CompatibilityBadge::Unsupported
    }
}

pub fn compatibility_note(entry: &WorkshopCatalogEntry) -> Option<String> {
    match (entry.sync_state, entry.project_type) {
        (WorkshopSyncState::MissingProjectFile, _) => Some(
            "The local Workshop folder is missing valid project metadata, so LWE cannot classify or import this item yet.".to_string(),
        ),
        (WorkshopSyncState::MissingPrimaryAsset, WorkshopProjectType::Video | WorkshopProjectType::Scene) => Some(
            "The project metadata was found, but the primary local asset is missing, so it cannot be projected into Library yet.".to_string(),
        ),
        (WorkshopSyncState::UnsupportedType, WorkshopProjectType::Web) => Some(
            "Web Workshop items are visible here, but the first release only supports video and scene imports.".to_string(),
        ),
        (WorkshopSyncState::UnsupportedType, _) => Some(
            "This Workshop item uses a project type that the first release does not import yet.".to_string(),
        ),
        _ => Some("This item is synchronized locally and available in the Library page.".to_string()),
    }
}
