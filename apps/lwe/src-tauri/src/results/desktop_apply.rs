#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopApplyResult {
    Applied { monitor_id: String, item_id: String },
    Cleared { monitor_id: String },
    MonitorNotFound { monitor_id: String },
    Unavailable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_apply_result_distinguishes_unavailable_from_known_failures() {
        let unavailable = DesktopApplyResult::Unavailable {
            reason: "Desktop persistence is not available yet".to_string(),
        };
        let missing = DesktopApplyResult::MonitorNotFound {
            monitor_id: "DISPLAY-2".to_string(),
        };

        assert!(matches!(
            unavailable,
            DesktopApplyResult::Unavailable { .. }
        ));
        assert!(matches!(
            missing,
            DesktopApplyResult::MonitorNotFound { .. }
        ));
    }
}
