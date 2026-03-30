use std::collections::BTreeMap;

use crate::services::monitor_service::MonitorDescriptor;

#[path = "desktop_apply.rs"]
mod desktop_apply_result;

pub use desktop_apply_result::DesktopApplyResult;

#[derive(Debug, Clone)]
pub struct DesktopPageResult {
    pub monitors: Vec<MonitorDescriptor>,
    pub assignments: BTreeMap<String, String>,
    pub monitor_discovery_issue: Option<String>,
    pub persistence_issue: Option<String>,
    pub assignments_available: bool,
    pub stale: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_page_result_tracks_unavailable_assignments_separately_from_empty_monitors() {
        let result = DesktopPageResult {
            monitors: Vec::new(),
            assignments: BTreeMap::new(),
            monitor_discovery_issue: None,
            persistence_issue: Some("Desktop persistence is not available yet".to_string()),
            assignments_available: false,
            stale: true,
        };

        assert!(result.monitors.is_empty());
        assert!(!result.assignments_available);
        assert!(result.persistence_issue.is_some());
        assert!(result.monitor_discovery_issue.is_none());
    }
}
