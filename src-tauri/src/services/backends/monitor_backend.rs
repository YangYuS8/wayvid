#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendMonitorDescriptor {
    pub id: String,
    pub backend_output_id: String,
    pub name: String,
    pub resolution: String,
}

pub enum BackendMonitorDiscovery {
    Known(Vec<BackendMonitorDescriptor>),
    Unavailable { reason: String },
}

pub trait MonitorBackend {
    fn list_monitors(&self) -> BackendMonitorDiscovery;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_descriptor_v1_stays_small_and_stable() {
        let monitor = BackendMonitorDescriptor {
            id: "eDP-1".to_string(),
            backend_output_id: "eDP-1".to_string(),
            name: "Built-in".to_string(),
            resolution: "2160x1440".to_string(),
        };

        assert_eq!(monitor.id, "eDP-1");
        assert_eq!(monitor.backend_output_id, "eDP-1");
        assert_eq!(monitor.name, "Built-in");
        assert_eq!(monitor.resolution, "2160x1440");
    }
}
