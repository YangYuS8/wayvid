use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistedDesktopAssignments {
    pub assignments: BTreeMap<String, String>,
}

pub trait PersistenceBackend {
    fn load_assignments(&self) -> Result<PersistedDesktopAssignments, String>;

    fn save_assignments(&self, assignments: &PersistedDesktopAssignments) -> Result<(), String>;

    fn clear_assignments(&self) -> Result<(), String>;
}
