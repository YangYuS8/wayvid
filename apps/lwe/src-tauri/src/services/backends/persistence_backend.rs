use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PersistedDesktopAssignments {
    pub assignments: BTreeMap<String, String>,
}

pub fn desktop_state_path() -> PathBuf {
    PathBuf::from("desktop-state.json")
}
