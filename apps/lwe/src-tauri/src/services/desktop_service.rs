use std::collections::BTreeMap;

use crate::results::desktop::{DesktopApplyResult, DesktopPageResult};
use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};
use crate::results::monitor_discovery::MonitorDiscoveryResult;
use crate::services::desktop_persistence_service::DesktopPersistenceService;
use crate::services::monitor_service::MonitorService;

pub struct DesktopService;

impl DesktopService {
    pub fn load_page() -> Result<DesktopPageResult, String> {
        let monitors = MonitorService::list_monitors();
        let assignments = DesktopPersistenceService::load_state();

        let (monitors, monitors_available, monitor_discovery_issue) = match monitors {
            MonitorDiscoveryResult::Known(monitors) => (monitors, true, None),
            MonitorDiscoveryResult::Unavailable { reason } => (Vec::new(), false, Some(reason)),
        };
        let (assignments, persistence_issue, assignments_available) = match assignments {
            DesktopPersistenceLoad::Loaded(assignments) => (assignments, None, true),
            DesktopPersistenceLoad::Unavailable { reason } => {
                (BTreeMap::new(), Some(reason), false)
            }
        };
        let stale = !monitors_available || !assignments_available;

        Ok(DesktopPageResult {
            monitors,
            assignments,
            monitors_available,
            monitor_discovery_issue,
            persistence_issue,
            assignments_available,
            stale,
        })
    }

    pub fn apply_to_monitor(monitor_id: &str, item_id: &str) -> Result<DesktopApplyResult, String> {
        let monitors = MonitorService::list_monitors();

        match MonitorService::resolve_specific_monitor(&monitors, monitor_id) {
            MonitorDiscoveryResult::Known(monitors) if monitors.is_empty() => {
                Ok(DesktopApplyResult::MonitorNotFound {
                    monitor_id: monitor_id.to_string(),
                })
            }
            MonitorDiscoveryResult::Known(_) => {
                match DesktopPersistenceService::save_assignment(monitor_id, item_id) {
                    DesktopPersistenceWrite::Saved => Ok(DesktopApplyResult::Applied {
                        monitor_id: monitor_id.to_string(),
                        item_id: item_id.to_string(),
                    }),
                    DesktopPersistenceWrite::Cleared => {
                        Ok(DesktopApplyResult::PersistenceUnavailable {
                            reason: "Desktop persistence returned a clear result while saving"
                                .to_string(),
                        })
                    }
                    DesktopPersistenceWrite::Unavailable { reason } => {
                        Ok(DesktopApplyResult::PersistenceUnavailable { reason })
                    }
                }
            }
            MonitorDiscoveryResult::Unavailable { reason } => {
                Ok(DesktopApplyResult::MonitorDiscoveryUnavailable { reason })
            }
        }
    }

    pub fn clear_monitor(monitor_id: &str) -> Result<DesktopApplyResult, String> {
        let monitors = MonitorService::list_monitors();

        match MonitorService::resolve_specific_monitor(&monitors, monitor_id) {
            MonitorDiscoveryResult::Known(monitors) if monitors.is_empty() => {
                Ok(DesktopApplyResult::MonitorNotFound {
                    monitor_id: monitor_id.to_string(),
                })
            }
            MonitorDiscoveryResult::Known(_) => {
                match DesktopPersistenceService::clear_assignment(monitor_id) {
                    DesktopPersistenceWrite::Cleared => Ok(DesktopApplyResult::Cleared {
                        monitor_id: monitor_id.to_string(),
                    }),
                    DesktopPersistenceWrite::Saved => {
                        Ok(DesktopApplyResult::PersistenceUnavailable {
                            reason: "Desktop persistence returned a save result while clearing"
                                .to_string(),
                        })
                    }
                    DesktopPersistenceWrite::Unavailable { reason } => {
                        Ok(DesktopApplyResult::PersistenceUnavailable { reason })
                    }
                }
            }
            MonitorDiscoveryResult::Unavailable { reason } => {
                Ok(DesktopApplyResult::MonitorDiscoveryUnavailable { reason })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::desktop::DesktopApplyResult;

    #[test]
    fn desktop_apply_flow_load_page_marks_state_unavailable_when_monitor_or_persistence_is_unavailable(
    ) {
        let result = DesktopService::load_page().unwrap();
        let DesktopPageResult {
            monitors_available,
            stale,
            assignments_available,
            ..
        } = result;

        assert!(stale);
        assert!(!monitors_available);
        assert!(!assignments_available);
    }

    #[test]
    fn desktop_apply_flow_apply_to_monitor_returns_unavailable_when_dependencies_are_unavailable() {
        let result = DesktopService::apply_to_monitor("DISPLAY-1", "wallpaper-1").unwrap();

        assert!(matches!(
            result,
            DesktopApplyResult::MonitorDiscoveryUnavailable { .. }
        ));
    }

    #[test]
    fn desktop_apply_flow_clear_monitor_returns_unavailable_when_dependencies_are_unavailable() {
        let result = DesktopService::clear_monitor("DISPLAY-1").unwrap();

        assert!(matches!(
            result,
            DesktopApplyResult::MonitorDiscoveryUnavailable { .. }
        ));
    }
}
