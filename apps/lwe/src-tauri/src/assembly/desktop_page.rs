use crate::models::{
    DesktopMonitorSummary, DesktopPageSnapshot, DesktopRestoreState, RuntimeStatus,
};
use crate::results::desktop::{DesktopPageResult, DesktopResolvedMonitorAssignment};

pub fn assemble_desktop_page(result: DesktopPageResult) -> DesktopPageSnapshot {
    let DesktopPageResult {
        monitors,
        assignments,
        resolved_assignments,
        library_item_assignments: _,
        restore_issues,
        monitors_available,
        monitor_discovery_issue,
        persistence_issue,
        assignments_available,
        stale,
    } = result;

    DesktopPageSnapshot {
        monitors: monitors
            .into_iter()
            .map(|monitor| {
                let monitor_id = monitor.id;

                DesktopMonitorSummary {
                    current_wallpaper_title: match resolved_assignments.get(&monitor_id) {
                        Some(DesktopResolvedMonitorAssignment::Restored { item_title, .. }) => {
                            Some(item_title.clone())
                        }
                        _ => assignments.get(&monitor_id).cloned(),
                    },
                    current_cover_path: None,
                    current_item_id: resolved_assignments.get(&monitor_id).map(|assignment| {
                        match assignment {
                            DesktopResolvedMonitorAssignment::Restored { item_id, .. }
                            | DesktopResolvedMonitorAssignment::MissingItem { item_id }
                            | DesktopResolvedMonitorAssignment::Unavailable { item_id, .. } => {
                                item_id.clone()
                            }
                        }
                    }),
                    display_name: monitor.name,
                    monitor_id: monitor_id.clone(),
                    resolution: monitor.resolution,
                    restore_state: resolved_assignments.get(&monitor_id).map(|assignment| {
                        match assignment {
                            DesktopResolvedMonitorAssignment::Restored { .. } => {
                                DesktopRestoreState::Restored
                            }
                            DesktopResolvedMonitorAssignment::MissingItem { .. } => {
                                DesktopRestoreState::MissingItem
                            }
                            DesktopResolvedMonitorAssignment::Unavailable { .. } => {
                                DesktopRestoreState::Unavailable
                            }
                        }
                    }),
                    restore_issue: resolved_assignments
                        .get(&monitor_id)
                        .and_then(|assignment| match assignment {
                            DesktopResolvedMonitorAssignment::Restored { .. } => None,
                            DesktopResolvedMonitorAssignment::MissingItem { item_id } => {
                                Some(format!(
                                    "Saved assignment references missing Library item {item_id}."
                                ))
                            }
                            DesktopResolvedMonitorAssignment::Unavailable { reason, .. } => {
                                Some(reason.clone())
                            }
                        }),
                    runtime_status: RuntimeStatus::Unsupported,
                }
            })
            .collect(),
        monitors_available,
        monitor_discovery_issue,
        persistence_issue,
        assignments_available,
        restore_issues,
        stale,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::models::RuntimeStatus;
    use crate::services::monitor_service::MonitorDescriptor;

    #[test]
    fn desktop_apply_flow_assembler_preserves_known_monitors_and_assignments() {
        let mut assignments = BTreeMap::new();
        assignments.insert("DISPLAY-1".to_string(), "Forest Scene".to_string());

        let snapshot = assemble_desktop_page(DesktopPageResult {
            monitors: vec![MonitorDescriptor {
                id: "DISPLAY-1".to_string(),
                name: "Primary".to_string(),
                resolution: "1920x1080".to_string(),
            }],
            assignments,
            resolved_assignments: BTreeMap::from([(
                "DISPLAY-1".to_string(),
                DesktopResolvedMonitorAssignment::Restored {
                    item_id: "Forest Scene".to_string(),
                    item_title: "Forest Scene".to_string(),
                },
            )]),
            library_item_assignments: BTreeMap::new(),
            restore_issues: Vec::new(),
            monitors_available: true,
            monitor_discovery_issue: None,
            persistence_issue: None,
            assignments_available: true,
            stale: false,
        });

        assert_eq!(snapshot.monitors.len(), 1);
        assert_eq!(snapshot.monitors[0].display_name, "Primary");
        assert_eq!(snapshot.monitors[0].resolution, "1920x1080");
        assert_eq!(
            snapshot.monitors[0].current_wallpaper_title.as_deref(),
            Some("Forest Scene")
        );
        assert!(!snapshot.stale);
        assert!(snapshot.assignments_available);
        assert!(snapshot.monitor_discovery_issue.is_none());
        assert!(snapshot.persistence_issue.is_none());
    }

    #[test]
    fn desktop_apply_flow_assembler_keeps_runtime_status_unsupported_when_only_assignments_are_unavailable(
    ) {
        let snapshot = assemble_desktop_page(DesktopPageResult {
            monitors: vec![MonitorDescriptor {
                id: "DISPLAY-1".to_string(),
                name: "Primary".to_string(),
                resolution: "1920x1080".to_string(),
            }],
            assignments: BTreeMap::new(),
            resolved_assignments: BTreeMap::new(),
            library_item_assignments: BTreeMap::new(),
            restore_issues: Vec::new(),
            monitors_available: true,
            monitor_discovery_issue: None,
            persistence_issue: Some("Desktop persistence is not available yet".to_string()),
            assignments_available: false,
            stale: true,
        });

        assert_eq!(snapshot.monitors.len(), 1);
        assert_eq!(
            snapshot.monitors[0].runtime_status,
            RuntimeStatus::Unsupported
        );
    }

    #[test]
    fn desktop_apply_flow_assembler_preserves_restore_state_and_page_issues() {
        let mut resolved_assignments = BTreeMap::new();
        resolved_assignments.insert(
            "DISPLAY-1".to_string(),
            crate::results::desktop::DesktopResolvedMonitorAssignment::MissingItem {
                item_id: "missing-item".to_string(),
            },
        );

        let snapshot = assemble_desktop_page(DesktopPageResult {
            monitors: vec![MonitorDescriptor {
                id: "DISPLAY-1".to_string(),
                name: "Primary".to_string(),
                resolution: "1920x1080".to_string(),
            }],
            assignments: BTreeMap::new(),
            resolved_assignments,
            library_item_assignments: BTreeMap::new(),
            restore_issues: vec![
                "Saved assignment for missing monitor DISPLAY-3 still points to Forest Scene (scene-7)."
                    .to_string(),
            ],
            monitors_available: true,
            monitor_discovery_issue: None,
            persistence_issue: None,
            assignments_available: true,
            stale: false,
        });

        assert_eq!(
            snapshot.monitors[0].current_item_id.as_deref(),
            Some("missing-item")
        );
        assert_eq!(
            snapshot.monitors[0].restore_state,
            Some(crate::models::DesktopRestoreState::MissingItem)
        );
        assert_eq!(snapshot.restore_issues.len(), 1);
    }
}
