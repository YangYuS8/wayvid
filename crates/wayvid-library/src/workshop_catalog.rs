use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkshopProjectType {
    Video,
    Scene,
    Web,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkshopSyncState {
    Synced,
    MissingProjectFile,
    MissingPrimaryAsset,
    UnsupportedType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopCatalogEntry {
    pub workshop_id: u64,
    pub title: String,
    pub project_type: WorkshopProjectType,
    pub project_dir: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub sync_state: WorkshopSyncState,
    pub supported_first_release: bool,
    pub library_item_id: Option<String>,
}

impl WorkshopCatalogEntry {
    pub fn has_cover(&self) -> bool {
        self.cover_path.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn bundled_cover_is_used_when_present() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 101,
            title: "Forest Scene".to_string(),
            project_type: WorkshopProjectType::Scene,
            project_dir: PathBuf::from("/tmp/431960/101"),
            cover_path: Some(PathBuf::from("/tmp/431960/101/preview.jpg")),
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("forest-101".to_string()),
        };

        assert!(entry.has_cover());
    }

    #[test]
    fn unsupported_web_item_has_no_cover_requirement() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 202,
            title: "Interactive Web".to_string(),
            project_type: WorkshopProjectType::Web,
            project_dir: PathBuf::from("/tmp/431960/202"),
            cover_path: None,
            sync_state: WorkshopSyncState::UnsupportedType,
            supported_first_release: false,
            library_item_id: None,
        };

        assert!(!entry.has_cover());
        assert!(!entry.supported_first_release);
    }
}
