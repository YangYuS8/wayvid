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

    #[test]
    fn placeholder_desktop_snapshot_is_marked_stale() {
        let snapshot = load_desktop_page().unwrap();

        assert!(snapshot.monitors.is_empty());
        assert!(snapshot.stale);
        assert_eq!(
            snapshot.monitor_discovery_issue.as_deref(),
            Some("Monitor discovery is not available yet")
        );
        assert_eq!(
            snapshot.persistence_issue.as_deref(),
            Some("Desktop persistence is not available yet")
        );
        assert!(!snapshot.assignments_available);
    }

    #[test]
    fn desktop_apply_flow_command_returns_failure_outcome_for_unavailable_apply() {
        let outcome =
            apply_library_item_to_monitor("DISPLAY-1".to_string(), "wallpaper-1".to_string())
                .unwrap();

        assert!(!outcome.ok);
        assert_eq!(
            outcome.message.as_deref(),
            Some("Monitor discovery is not available yet")
        );
    }

    #[test]
    fn desktop_apply_flow_command_returns_failure_outcome_for_unavailable_clear() {
        let outcome = clear_library_item_from_monitor("DISPLAY-1".to_string()).unwrap();

        assert!(!outcome.ok);
        assert_eq!(
            outcome.message.as_deref(),
            Some("Monitor discovery is not available yet")
        );
    }
}
