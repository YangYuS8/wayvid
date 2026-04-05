use crate::models::{WorkshopAgeRating, WorkshopOnlineItemType};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PersistedSettings {
    pub language: String,
    pub theme: String,
    pub launch_on_login: bool,
    pub steam_web_api_key: String,
    pub workshop_query: String,
    pub workshop_age_ratings: Vec<WorkshopAgeRating>,
    pub workshop_item_types: Vec<WorkshopOnlineItemType>,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            language: "system".to_string(),
            theme: "system".to_string(),
            launch_on_login: false,
            steam_web_api_key: String::new(),
            workshop_query: String::new(),
            workshop_age_ratings: vec![WorkshopAgeRating::G, WorkshopAgeRating::Pg13],
            workshop_item_types: vec![
                WorkshopOnlineItemType::Video,
                WorkshopOnlineItemType::Scene,
                WorkshopOnlineItemType::Web,
                WorkshopOnlineItemType::Application,
            ],
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
