use std::collections::BTreeMap;

pub type ScenePropertyValues = BTreeMap<String, serde_json::Value>;
pub type PersistedSceneProperties = BTreeMap<String, BTreeMap<String, ScenePropertyValues>>;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PersistedSessionState {
    pub assignments: BTreeMap<String, String>,
    pub scene_properties: PersistedSceneProperties,
}
