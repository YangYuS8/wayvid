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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilitySummaryModel {
    pub badge: CompatibilityBadge,
    pub reason_code: String,
    pub summary_copy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityExplanationModel {
    pub badge: CompatibilityBadge,
    pub reason_code: String,
    pub summary_copy: String,
    pub headline: String,
    pub detail: String,
    pub next_step: crate::results::compatibility::CompatibilityNextStep,
    pub next_step_copy: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRestoreState {
    Restored,
    MissingItem,
    Unavailable,
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
    pub compatibility: CompatibilitySummaryModel,
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
    pub compatibility: CompatibilityExplanationModel,
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
    pub compatibility: CompatibilitySummaryModel,
    pub favorite: bool,
    pub assigned_monitor_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPageSnapshot {
    pub items: Vec<LibraryItemSummary>,
    pub selected_item_id: Option<String>,
    pub monitors_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor_discovery_issue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop_assignment_issue: Option<String>,
    pub desktop_assignments_available: bool,
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
    pub compatibility: CompatibilityExplanationModel,
    pub monitors_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor_discovery_issue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop_assignment_issue: Option<String>,
    pub desktop_assignments_available: bool,
    pub assigned_monitor_labels: Vec<String>,
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
    pub current_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore_state: Option<DesktopRestoreState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore_issue: Option<String>,
    pub runtime_status: RuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopPageSnapshot {
    pub monitors: Vec<DesktopMonitorSummary>,
    pub monitors_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor_discovery_issue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_issue: Option<String>,
    pub assignments_available: bool,
    pub restore_issues: Vec<String>,
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
    use crate::results::compatibility::CompatibilityNextStep;

    fn summary_compatibility() -> CompatibilitySummaryModel {
        CompatibilitySummaryModel {
            badge: CompatibilityBadge::FullySupported,
            reason_code: "ready_for_library".to_string(),
            summary_copy: "Ready to use".to_string(),
        }
    }

    #[test]
    fn workshop_item_summary_uses_cover_or_placeholder_shape() {
        let item = WorkshopItemSummary {
            id: "42".to_string(),
            title: "Forest Scene".to_string(),
            item_type: ItemType::Scene,
            cover_path: None,
            sync_status: WorkshopSyncStatus::Synced,
            compatibility: summary_compatibility(),
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
            compatibility: summary_compatibility(),
        };

        let value = serde_json::to_value(&item).unwrap();

        assert_eq!(value["id"], "9007199254740993");
        assert_eq!(value["itemType"], "scene");
        assert_eq!(value["syncStatus"], "synced");
        assert_eq!(value["compatibility"]["badge"], "fully_supported");
        assert_eq!(value["compatibility"]["reasonCode"], "ready_for_library");
    }

    #[test]
    fn compatibility_models_serialize_explanation_payload() {
        let item = WorkshopItemDetail {
            id: "42".to_string(),
            title: "Forest Scene".to_string(),
            item_type: ItemType::Scene,
            cover_path: None,
            sync_status: WorkshopSyncStatus::Synced,
            compatibility: CompatibilityExplanationModel {
                badge: CompatibilityBadge::FullySupported,
                reason_code: "ready_for_library".to_string(),
                summary_copy: "Ready to use".to_string(),
                headline: "Ready to use".to_string(),
                detail:
                    "This item is synchronized locally and available for Library and desktop use."
                        .to_string(),
                next_step: CompatibilityNextStep::None,
                next_step_copy: None,
            },
            tags: Vec::new(),
            description: None,
        };

        let value = serde_json::to_value(&item).unwrap();

        assert_eq!(value["compatibility"]["headline"], "Ready to use");
        assert_eq!(value["compatibility"]["summaryCopy"], "Ready to use");
        assert_eq!(value["compatibility"]["nextStep"], "none");
        assert!(value["compatibility"]["nextStepCopy"].is_null());
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

    #[test]
    fn desktop_apply_flow_models_serialize_restore_state_and_assignment_labels() {
        let snapshot = DesktopPageSnapshot {
            monitors: vec![DesktopMonitorSummary {
                monitor_id: "DISPLAY-1".to_string(),
                display_name: "Primary".to_string(),
                resolution: "1920x1080".to_string(),
                current_wallpaper_title: None,
                current_cover_path: None,
                current_item_id: Some("scene-7".to_string()),
                restore_state: Some(DesktopRestoreState::Restored),
                restore_issue: None,
                runtime_status: RuntimeStatus::Unsupported,
            }],
            monitors_available: true,
            monitor_discovery_issue: None,
            persistence_issue: None,
            assignments_available: true,
            restore_issues: vec![
                "Saved assignment for missing monitor DISPLAY-2 still points to Forest Scene (scene-7)."
                    .to_string(),
            ],
            stale: false,
        };

        let library_item = LibraryItemSummary {
            id: "scene-7".to_string(),
            title: "Forest Scene".to_string(),
            item_type: ItemType::Scene,
            cover_path: None,
            source: LibrarySource::Workshop,
            compatibility: summary_compatibility(),
            favorite: false,
            assigned_monitor_labels: vec!["Primary".to_string()],
        };

        let desktop_value = serde_json::to_value(&snapshot).unwrap();
        let library_value = serde_json::to_value(&library_item).unwrap();

        assert_eq!(desktop_value["monitors"][0]["restoreState"], "restored");
        assert_eq!(desktop_value["restoreIssues"][0], "Saved assignment for missing monitor DISPLAY-2 still points to Forest Scene (scene-7).");
        assert_eq!(library_value["assignedMonitorLabels"][0], "Primary");
    }
}
