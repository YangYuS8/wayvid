#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopApplyResult {
    Applied { monitor_id: String, item_id: String },
    Cleared { monitor_id: String },
    MonitorNotFound { monitor_id: String },
    MonitorDiscoveryUnavailable { reason: String },
    PersistenceUnavailable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_apply_result_distinguishes_unavailable_from_known_failures() {
        let applied = DesktopApplyResult::Applied {
            monitor_id: "DISPLAY-1".to_string(),
            item_id: "scene-7".to_string(),
        };
        let discovery_unavailable = DesktopApplyResult::MonitorDiscoveryUnavailable {
            reason: "Desktop persistence is not available yet".to_string(),
        };
        let persistence_unavailable = DesktopApplyResult::PersistenceUnavailable {
            reason: "Desktop persistence is not available yet".to_string(),
        };
        let missing = DesktopApplyResult::MonitorNotFound {
            monitor_id: "DISPLAY-2".to_string(),
        };

        assert!(matches!(applied, DesktopApplyResult::Applied { .. }));
        assert!(matches!(
            discovery_unavailable,
            DesktopApplyResult::MonitorDiscoveryUnavailable { .. }
        ));
        assert!(matches!(
            persistence_unavailable,
            DesktopApplyResult::PersistenceUnavailable { .. }
        ));
        assert!(matches!(
            missing,
            DesktopApplyResult::MonitorNotFound { .. }
        ));
    }
}
