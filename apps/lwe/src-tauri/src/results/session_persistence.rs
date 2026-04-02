use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistedSessionState {
    pub assignments: BTreeMap<String, String>,
}
