use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Video,
    Scene,
    Web,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkshopSyncStatus {
    Synced,
    MissingProject,
    MissingAsset,
    UnsupportedType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityBadge {
    FullySupported,
    PartiallySupported,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibrarySource {
    Local,
    Workshop,
    Core,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    Running,
    Idle,
    Unsupported,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppShellSnapshot {
    pub app_name: String,
    pub code_name: String,
    pub steam_available: bool,
    pub library_count: Option<usize>,
    pub workshop_synced_count: Option<usize>,
    pub monitor_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopItemSummary {
    pub id: String,
    pub title: String,
    pub item_type: ItemType,
    pub cover_path: Option<String>,
    pub sync_status: WorkshopSyncStatus,
    pub compatibility_badge: CompatibilityBadge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopPageSnapshot {
    pub items: Vec<WorkshopItemSummary>,
    pub selected_item_id: Option<String>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopItemDetail {
    pub id: String,
    pub title: String,
    pub item_type: ItemType,
    pub cover_path: Option<String>,
    pub sync_status: WorkshopSyncStatus,
    pub compatibility_badge: CompatibilityBadge,
    pub compatibility_note: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemSummary {
    pub id: String,
    pub title: String,
    pub item_type: ItemType,
    pub cover_path: Option<String>,
    pub source: LibrarySource,
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
    pub item_type: ItemType,
    pub cover_path: Option<String>,
    pub source: LibrarySource,
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
    pub runtime_status: RuntimeStatus,
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
    use crate::action_outcome::AppShellPatch;

    #[test]
    fn workshop_item_summary_uses_cover_or_placeholder_shape() {
        let item = WorkshopItemSummary {
            id: "42".to_string(),
            title: "Forest Scene".to_string(),
            item_type: ItemType::Scene,
            cover_path: None,
            sync_status: WorkshopSyncStatus::Synced,
            compatibility_badge: CompatibilityBadge::FullySupported,
        };

        assert_eq!(item.id, "42");
        assert_eq!(item.item_type, ItemType::Scene);
        assert!(item.cover_path.is_none());
    }

    #[test]
    fn models_serialize_using_safe_ids_and_constrained_labels() {
        let item = WorkshopItemSummary {
            id: "9007199254740993".to_string(),
            title: "Forest Scene".to_string(),
            item_type: ItemType::Scene,
            cover_path: None,
            sync_status: WorkshopSyncStatus::Synced,
            compatibility_badge: CompatibilityBadge::FullySupported,
        };

        let value = serde_json::to_value(&item).unwrap();

        assert_eq!(value["id"], "9007199254740993");
        assert_eq!(value["itemType"], "scene");
        assert_eq!(value["syncStatus"], "synced");
        assert_eq!(value["compatibilityBadge"], "fully_supported");
    }

    #[test]
    fn app_shell_patch_omits_absent_counts() {
        let patch = AppShellPatch {
            workshop_synced_count: Some(2),
            library_count: None,
            monitor_count: None,
        };

        let value = serde_json::to_value(&patch).unwrap();

        assert_eq!(value["workshopSyncedCount"], 2);
        assert!(value.get("libraryCount").is_none());
        assert!(value.get("monitorCount").is_none());
    }

    #[test]
    fn app_shell_snapshot_allows_unknown_counts() {
        let snapshot = AppShellSnapshot {
            app_name: "LWE".to_string(),
            code_name: "lwe".to_string(),
            steam_available: false,
            library_count: None,
            workshop_synced_count: None,
            monitor_count: None,
        };

        let value = serde_json::to_value(&snapshot).unwrap();

        assert!(value["libraryCount"].is_null());
        assert!(value["workshopSyncedCount"].is_null());
        assert!(value["monitorCount"].is_null());
    }
}
