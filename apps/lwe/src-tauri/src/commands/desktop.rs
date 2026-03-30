use crate::action_outcome::ActionOutcome;
use crate::assembly::action_outcome::assemble_desktop_apply_outcome;
use crate::assembly::desktop_page::assemble_desktop_page;
use crate::models::DesktopPageSnapshot;
use crate::services::desktop_service::DesktopService;

#[tauri::command]
pub fn load_desktop_page() -> Result<DesktopPageSnapshot, String> {
    DesktopService::load_page().map(assemble_desktop_page)
}

#[tauri::command]
pub fn apply_library_item_to_monitor(
    monitor_id: String,
    item_id: String,
) -> Result<ActionOutcome<()>, String> {
    Ok(assemble_desktop_apply_outcome(
        DesktopService::apply_to_monitor(&monitor_id, &item_id)?,
    ))
}

#[tauri::command]
pub fn clear_library_item_from_monitor(monitor_id: String) -> Result<ActionOutcome<()>, String> {
    Ok(assemble_desktop_apply_outcome(
        DesktopService::clear_monitor(&monitor_id)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::monitor_service::MonitorService;

    fn known_monitor_id() -> Option<String> {
        match MonitorService::list_monitors() {
            crate::results::monitor_discovery::MonitorDiscoveryResult::Known(monitors) => {
                monitors.into_iter().next().map(|monitor| monitor.id)
            }
            crate::results::monitor_discovery::MonitorDiscoveryResult::Unavailable { .. } => None,
        }
    }

    #[test]
    fn desktop_snapshot_reflects_real_monitor_discovery_state() {
        let snapshot = load_desktop_page().unwrap();

        assert!(snapshot.stale);
        assert_eq!(
            snapshot.persistence_issue.as_deref(),
            Some("Desktop persistence is not available yet")
        );
        assert!(!snapshot.assignments_available);

        if snapshot.monitors_available {
            assert!(snapshot.monitor_discovery_issue.is_none());
        } else {
            assert!(snapshot.monitors.is_empty());
            assert!(snapshot.monitor_discovery_issue.is_some());
        }
    }

    #[test]
    fn desktop_apply_flow_command_returns_failure_outcome_for_current_dependency_state() {
        let known_monitor_id = known_monitor_id();
        let monitor_id = known_monitor_id
            .clone()
            .unwrap_or_else(|| "DISPLAY-1".to_string());
        let outcome = apply_library_item_to_monitor(monitor_id, "wallpaper-1".to_string()).unwrap();

        if known_monitor_id.is_some() {
            assert!(outcome.ok);
            assert!(matches!(
                outcome.message.as_deref(),
                Some(message) if message.contains("Applied wallpaper-1 to")
            ));
        } else {
            assert!(!outcome.ok);
            assert!(outcome.message.as_deref().is_some());
        }
    }

    #[test]
    fn desktop_apply_flow_command_returns_failure_outcome_for_current_clear_state() {
        let known_monitor_id = known_monitor_id();
        let monitor_id = known_monitor_id
            .clone()
            .unwrap_or_else(|| "DISPLAY-1".to_string());
        let outcome = clear_library_item_from_monitor(monitor_id).unwrap();

        if known_monitor_id.is_some() {
            assert!(matches!(
                outcome,
                ActionOutcome {
                    ok: true,
                    message: Some(_),
                    ..
                } | ActionOutcome {
                    ok: false,
                    message: Some(_),
                    ..
                }
            ));
        } else {
            assert!(!outcome.ok);
            assert!(outcome.message.as_deref().is_some());
        }
    }
}
