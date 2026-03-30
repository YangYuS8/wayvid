#[derive(Debug, Clone)]
pub struct MonitorDescriptor {
    pub id: String,
    pub name: String,
}

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
}
