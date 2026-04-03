#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PersistedSettings {
    pub language: String,
    pub theme: String,
    pub launch_on_login: bool,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            language: "system".to_string(),
            theme: "system".to_string(),
            launch_on_login: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceLoad {
    Loaded(PersistedSettings),
    Unavailable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceWrite {
    Saved,
    Unavailable { reason: String },
}
