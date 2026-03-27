use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppShellSnapshot {
    pub app_name: String,
    pub code_name: String,
    pub steam_available: bool,
    pub library_count: usize,
    pub workshop_synced_count: usize,
    pub monitor_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopItemSummary {
    pub id: u64,
    pub title: String,
    pub item_type: String,
    pub cover_path: Option<String>,
    pub sync_status: String,
    pub compatibility_badge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopPageSnapshot {
    pub items: Vec<WorkshopItemSummary>,
    pub selected_item_id: Option<u64>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopItemDetail {
    pub id: u64,
    pub title: String,
    pub item_type: String,
    pub cover_path: Option<String>,
    pub sync_status: String,
    pub compatibility_badge: String,
    pub compatibility_note: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemSummary {
    pub id: String,
    pub title: String,
    pub item_type: String,
    pub cover_path: Option<String>,
    pub source: String,
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPageSnapshot {
    pub items: Vec<LibraryItemSummary>,
    pub selected_item_id: Option<String>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemDetail {
    pub id: String,
    pub title: String,
    pub item_type: String,
    pub cover_path: Option<String>,
    pub source: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopMonitorSummary {
    pub monitor_id: String,
    pub display_name: String,
    pub resolution: String,
    pub current_wallpaper_title: Option<String>,
    pub current_cover_path: Option<String>,
    pub runtime_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopPageSnapshot {
    pub monitors: Vec<DesktopMonitorSummary>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPageSnapshot {
    pub language: String,
    pub theme: String,
    pub steam_required: bool,
    pub stale: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workshop_item_summary_uses_cover_or_placeholder_shape() {
        let item = WorkshopItemSummary {
            id: 42,
            title: "Forest Scene".to_string(),
            item_type: "scene".to_string(),
            cover_path: None,
            sync_status: "synced".to_string(),
            compatibility_badge: "Fully Supported".to_string(),
        };

        assert_eq!(item.item_type, "scene");
        assert!(item.cover_path.is_none());
    }
}
