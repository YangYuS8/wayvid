use crate::models::{DesktopMonitorSummary, DesktopPageSnapshot, RuntimeStatus};
use crate::results::desktop::DesktopPageResult;

pub fn assemble_desktop_page(result: DesktopPageResult) -> DesktopPageSnapshot {
    let DesktopPageResult {
        monitors,
        assignments,
        monitors_available: _,
        monitor_discovery_issue,
        persistence_issue,
        assignments_available,
        stale,
    } = result;

    DesktopPageSnapshot {
        monitors: monitors
            .into_iter()
            .map(|monitor| DesktopMonitorSummary {
                current_wallpaper_title: assignments.get(&monitor.id).cloned(),
                current_cover_path: None,
                display_name: monitor.name,
                monitor_id: monitor.id,
                resolution: "Unknown".to_string(),
                runtime_status: if assignments_available {
                    RuntimeStatus::Unsupported
                } else {
                    RuntimeStatus::Error
                },
            })
            .collect(),
        monitor_discovery_issue,
        persistence_issue,
        assignments_available,
        stale,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::services::monitor_service::MonitorDescriptor;

    #[test]
    fn desktop_apply_flow_assembler_preserves_known_monitors_and_assignments() {
        let mut assignments = BTreeMap::new();
        assignments.insert("DISPLAY-1".to_string(), "Forest Scene".to_string());

        let snapshot = assemble_desktop_page(DesktopPageResult {
            monitors: vec![MonitorDescriptor {
                id: "DISPLAY-1".to_string(),
                name: "Primary".to_string(),
            }],
            assignments,
            monitors_available: true,
            monitor_discovery_issue: None,
            persistence_issue: None,
            assignments_available: true,
            stale: false,
        });

        assert_eq!(snapshot.monitors.len(), 1);
        assert_eq!(snapshot.monitors[0].display_name, "Primary");
        assert_eq!(
            snapshot.monitors[0].current_wallpaper_title.as_deref(),
            Some("Forest Scene")
        );
        assert!(!snapshot.stale);
        assert!(snapshot.assignments_available);
        assert!(snapshot.monitor_discovery_issue.is_none());
        assert!(snapshot.persistence_issue.is_none());
    }
}
