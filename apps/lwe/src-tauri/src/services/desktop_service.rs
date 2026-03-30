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
    use crate::results::monitor_discovery::MonitorDiscoveryResult;
    use crate::services::monitor_service::MonitorService;

    fn known_monitor_id() -> Option<String> {
        match MonitorService::list_monitors() {
            MonitorDiscoveryResult::Known(monitors) => {
                monitors.into_iter().next().map(|monitor| monitor.id)
            }
            MonitorDiscoveryResult::Unavailable { .. } => None,
        }
    }

    #[test]
    fn desktop_apply_flow_load_page_reflects_real_monitor_and_persistence_availability() {
        let result = DesktopService::load_page().unwrap();
        let DesktopPageResult {
            monitors_available,
            stale,
            assignments_available,
            monitor_discovery_issue,
            persistence_issue,
            monitors,
            ..
        } = result;

        assert!(stale);
        assert!(!assignments_available);

        if monitors_available {
            assert!(monitor_discovery_issue.is_none());
        } else {
            assert!(monitors.is_empty());
            assert!(monitor_discovery_issue.is_some());
        }

        assert!(!assignments_available);
        assert_eq!(
            persistence_issue.as_deref(),
            Some("Desktop persistence is not available yet")
        );
    }

    #[test]
    fn desktop_apply_flow_apply_to_monitor_reflects_current_monitor_discovery_state() {
        let Some(monitor_id) = known_monitor_id() else {
            let result = DesktopService::apply_to_monitor("DISPLAY-1", "wallpaper-1").unwrap();

            assert!(matches!(
                result,
                DesktopApplyResult::MonitorDiscoveryUnavailable { .. }
            ));
            return;
        };

        let result = DesktopService::apply_to_monitor(&monitor_id, "wallpaper-1").unwrap();

        assert!(matches!(
            result,
            DesktopApplyResult::PersistenceUnavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));
    }

    #[test]
    fn desktop_apply_flow_clear_monitor_reflects_current_monitor_discovery_state() {
        let Some(monitor_id) = known_monitor_id() else {
            let result = DesktopService::clear_monitor("DISPLAY-1").unwrap();

            assert!(matches!(
                result,
                DesktopApplyResult::MonitorDiscoveryUnavailable { .. }
            ));
            return;
        };

        let result = DesktopService::clear_monitor(&monitor_id).unwrap();

        assert!(matches!(
            result,
            DesktopApplyResult::PersistenceUnavailable { reason }
                if reason == "Desktop persistence is not available yet"
        ));
    }
}
