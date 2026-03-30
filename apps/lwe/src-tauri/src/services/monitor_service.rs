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

    pub fn resolve_specific_monitor<'a>(
        monitors: &'a [MonitorDescriptor],
        monitor_id: &str,
    ) -> Option<&'a MonitorDescriptor> {
        monitors.iter().find(|monitor| monitor.id == monitor_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_monitors_returns_structured_result() {
        let result = MonitorService::list_monitors();

        assert!(matches!(
            result,
            MonitorDiscoveryResult::Known(_) | MonitorDiscoveryResult::Unavailable { .. }
        ));
    }

    #[test]
    fn resolve_specific_monitor_uses_known_monitor_slice() {
        let monitors = vec![
            MonitorDescriptor {
                id: "DISPLAY-1".to_string(),
                name: "Primary".to_string(),
            },
            MonitorDescriptor {
                id: "DISPLAY-2".to_string(),
                name: "Secondary".to_string(),
            },
        ];

        let resolved = MonitorService::resolve_specific_monitor(&monitors, "DISPLAY-2");

        assert!(matches!(resolved, Some(monitor) if monitor.name == "Secondary"));
        assert!(MonitorService::resolve_specific_monitor(&monitors, "DISPLAY-3").is_none());
    }
}
