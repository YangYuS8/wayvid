use crate::results::monitor_discovery::MonitorDiscoveryResult;

#[derive(Debug, Clone)]
pub struct MonitorDescriptor {
    pub id: String,
    pub name: String,
}

pub struct MonitorService;

impl MonitorService {
    pub fn list_monitors() -> MonitorDiscoveryResult {
        MonitorDiscoveryResult::Unavailable {
            reason: "Monitor discovery is not available yet".to_string(),
        }
    }

    pub fn resolve_specific_monitor(
        monitors: &MonitorDiscoveryResult,
        monitor_id: &str,
    ) -> MonitorDiscoveryResult {
        match monitors {
            MonitorDiscoveryResult::Known(monitors) => MonitorDiscoveryResult::Known(
                monitors
                    .iter()
                    .filter(|monitor| monitor.id == monitor_id)
                    .cloned()
                    .collect(),
            ),
            MonitorDiscoveryResult::Unavailable { reason } => MonitorDiscoveryResult::Unavailable {
                reason: reason.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_monitors_reports_placeholder_unavailable_state() {
        let result = MonitorService::list_monitors();

        assert!(matches!(
            result,
            MonitorDiscoveryResult::Unavailable { reason }
                if reason == "Monitor discovery is not available yet"
        ));
    }

    #[test]
    fn resolve_specific_monitor_preserves_discovery_state() {
        let known_monitors = MonitorDiscoveryResult::Known(vec![
            MonitorDescriptor {
                id: "DISPLAY-1".to_string(),
                name: "Primary".to_string(),
            },
            MonitorDescriptor {
                id: "DISPLAY-2".to_string(),
                name: "Secondary".to_string(),
            },
        ]);

        let unavailable = MonitorDiscoveryResult::Unavailable {
            reason: "Monitor discovery is not available yet".to_string(),
        };

        let resolved = MonitorService::resolve_specific_monitor(&known_monitors, "DISPLAY-2");
        let missing = MonitorService::resolve_specific_monitor(&known_monitors, "DISPLAY-3");
        let unresolved = MonitorService::resolve_specific_monitor(&unavailable, "DISPLAY-2");

        assert!(matches!(
            resolved,
            MonitorDiscoveryResult::Known(monitors)
                if monitors.len() == 1 && monitors[0].name == "Secondary"
        ));
        assert!(matches!(missing, MonitorDiscoveryResult::Known(monitors) if monitors.is_empty()));
        assert!(matches!(
            unresolved,
            MonitorDiscoveryResult::Unavailable { reason }
                if reason == "Monitor discovery is not available yet"
        ));
    }
}
