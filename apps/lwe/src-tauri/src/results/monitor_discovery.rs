use crate::services::monitor_service::MonitorDescriptor;

#[derive(Debug, Clone)]
pub enum MonitorDiscoveryResult {
    Known(Vec<MonitorDescriptor>),
    Unavailable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_discovery_result_distinguishes_known_empty_from_unavailable() {
        let known_empty = MonitorDiscoveryResult::Known(Vec::new());
        let unavailable = MonitorDiscoveryResult::Unavailable {
            reason: "discovery unavailable".to_string(),
        };

        assert!(matches!(known_empty, MonitorDiscoveryResult::Known(_)));
        assert!(matches!(
            unavailable,
            MonitorDiscoveryResult::Unavailable { .. }
        ));
    }

    #[test]
    fn monitor_discovery_result_uses_service_monitor_descriptor_type() {
        let descriptor = crate::services::monitor_service::MonitorDescriptor {
            id: "DISPLAY-1".to_string(),
            name: "Primary".to_string(),
            resolution: "1920x1080".to_string(),
        };

        let result = MonitorDiscoveryResult::Known(vec![descriptor]);

        assert!(matches!(result, MonitorDiscoveryResult::Known(monitors) if monitors.len() == 1));
    }
}
