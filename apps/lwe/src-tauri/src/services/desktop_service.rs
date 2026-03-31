use std::collections::BTreeMap;

use crate::results::desktop::{
    DesktopApplyResult, DesktopPageResult, DesktopResolvedMonitorAssignment,
};
use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};
use crate::results::library::LibraryProjection;
use crate::results::monitor_discovery::MonitorDiscoveryResult;
use crate::services::desktop_persistence_service::DesktopPersistenceService;
use crate::services::library_service::LibraryService;
use crate::services::monitor_service::MonitorService;

pub(crate) const LIBRARY_RESOLUTION_ISSUE_PREFIX: &str =
    "Unable to resolve desktop items against the current Library snapshot:";

pub struct DesktopService;

impl DesktopService {
    pub(crate) fn library_resolution_issue(reason: &str) -> String {
        format!("{LIBRARY_RESOLUTION_ISSUE_PREFIX} {reason}")
    }

    pub fn load_page() -> Result<DesktopPageResult, String> {
        Self::load_page_with_projection(LibraryService::load_projection())
    }

    pub(crate) fn load_page_with_projection(
        library_projection: Result<LibraryProjection, String>,
    ) -> Result<DesktopPageResult, String> {
        let monitors = MonitorService::list_monitors();
        let assignments = match DesktopPersistenceService::for_user_path() {
            Ok(service) => service.load_state(),
            Err(reason) => DesktopPersistenceLoad::Unavailable { reason },
        };

        Ok(Self::build_page_result(
            monitors,
            assignments,
            library_projection,
        ))
    }

    fn library_item_titles(projection: LibraryProjection) -> BTreeMap<String, String> {
        projection
            .entries
            .into_iter()
            .map(|entry| {
                (
                    entry.entry.library_item_id.unwrap_or_default(),
                    entry.entry.title,
                )
            })
            .collect()
    }

    fn assignment_monitor_label(
        monitor_names: &BTreeMap<String, String>,
        monitors_available: bool,
        monitor_id: &str,
    ) -> String {
        if let Some(name) = monitor_names.get(monitor_id) {
            name.clone()
        } else if monitors_available {
            format!("{monitor_id} (missing)")
        } else {
            format!("{monitor_id} (unavailable)")
        }
    }

    fn build_page_result(
        monitors: MonitorDiscoveryResult,
        assignments: DesktopPersistenceLoad,
        library_projection: Result<LibraryProjection, String>,
    ) -> DesktopPageResult {
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
        let library_item_titles = library_projection.map(Self::library_item_titles);
        let mut resolved_assignments = BTreeMap::new();
        let mut library_item_assignments = BTreeMap::new();
        let mut restore_issues = Vec::new();
        let monitor_names = monitors
            .iter()
            .map(|monitor| (monitor.id.clone(), monitor.name.clone()))
            .collect::<BTreeMap<_, _>>();

        if let Err(reason) = &library_item_titles {
            restore_issues.push(Self::library_resolution_issue(reason));
        }

        for (monitor_id, item_id) in &assignments {
            let monitor_known = monitor_names.contains_key(monitor_id);
            let monitor_label =
                Self::assignment_monitor_label(&monitor_names, monitors_available, monitor_id);

            match &library_item_titles {
                Ok(library_items) => {
                    if let Some(item_title) = library_items.get(item_id) {
                        library_item_assignments
                            .entry(item_id.clone())
                            .or_insert_with(Vec::new)
                            .push(monitor_label.clone());

                        if monitor_known {
                            resolved_assignments.insert(
                                monitor_id.clone(),
                                DesktopResolvedMonitorAssignment::Restored {
                                    item_id: item_id.clone(),
                                    item_title: item_title.clone(),
                                },
                            );
                        } else if !monitors_available {
                            resolved_assignments.insert(
                                monitor_id.clone(),
                                DesktopResolvedMonitorAssignment::Unavailable {
                                    item_id: item_id.clone(),
                                    item_title: Some(item_title.clone()),
                                    reason: format!(
                                        "Saved assignment for monitor {monitor_id} could not be verified because monitor discovery is unavailable: {}.",
                                        monitor_discovery_issue
                                            .as_deref()
                                            .unwrap_or("Unknown monitor discovery failure")
                                    ),
                                },
                            );
                        } else if monitors_available {
                            resolved_assignments.insert(
                                monitor_id.clone(),
                                DesktopResolvedMonitorAssignment::MissingMonitor {
                                    item_id: item_id.clone(),
                                    item_title: Some(item_title.clone()),
                                },
                            );
                            restore_issues.push(format!(
                                "Saved assignment for missing monitor {monitor_id} still points to {item_title} ({item_id})."
                            ));
                        }
                    } else if monitor_known {
                        resolved_assignments.insert(
                            monitor_id.clone(),
                            DesktopResolvedMonitorAssignment::MissingItem {
                                item_id: item_id.clone(),
                            },
                        );
                    } else if !monitors_available {
                        resolved_assignments.insert(
                            monitor_id.clone(),
                            DesktopResolvedMonitorAssignment::Unavailable {
                                item_id: item_id.clone(),
                                item_title: None,
                                reason: format!(
                                    "Saved assignment for monitor {monitor_id} could not be verified because monitor discovery is unavailable and the referenced Library item {item_id} is missing: {}.",
                                    monitor_discovery_issue
                                        .as_deref()
                                        .unwrap_or("Unknown monitor discovery failure")
                                ),
                            },
                        );
                    } else if monitors_available {
                        resolved_assignments.insert(
                            monitor_id.clone(),
                            DesktopResolvedMonitorAssignment::MissingMonitor {
                                item_id: item_id.clone(),
                                item_title: None,
                            },
                        );
                        restore_issues.push(format!(
                            "Saved assignment for missing monitor {monitor_id} references missing item {item_id}."
                        ));
                    }
                }
                Err(reason) if monitor_known => {
                    resolved_assignments.insert(
                        monitor_id.clone(),
                        DesktopResolvedMonitorAssignment::Unavailable {
                            item_id: item_id.clone(),
                            item_title: None,
                            reason: reason.clone(),
                        },
                    );
                }
                Err(reason) if !monitors_available => {
                    resolved_assignments.insert(
                        monitor_id.clone(),
                        DesktopResolvedMonitorAssignment::Unavailable {
                            item_id: item_id.clone(),
                            item_title: None,
                            reason: format!(
                                "Saved assignment for monitor {monitor_id} could not be verified because monitor discovery is unavailable while the Library snapshot could not be resolved: {reason}"
                            ),
                        },
                    );
                }
                Err(_) => {}
            }
        }

        let stale = !monitors_available
            || !assignments_available
            || library_item_titles.is_err()
            || !restore_issues.is_empty();

        DesktopPageResult {
            monitors,
            assignments,
            resolved_assignments,
            library_item_assignments,
            restore_issues,
            monitors_available,
            monitor_discovery_issue,
            persistence_issue,
            assignments_available,
            stale,
        }
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
                let persistence = match DesktopPersistenceService::for_user_path() {
                    Ok(service) => service,
                    Err(reason) => {
                        return Ok(DesktopApplyResult::PersistenceUnavailable { reason });
                    }
                };

                match persistence.save_assignment(monitor_id, item_id) {
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
                let persistence = match DesktopPersistenceService::for_user_path() {
                    Ok(service) => service,
                    Err(reason) => {
                        return Ok(DesktopApplyResult::PersistenceUnavailable { reason });
                    }
                };

                match persistence.clear_assignment(monitor_id) {
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
    use crate::results::desktop::DesktopResolvedMonitorAssignment;
    use crate::results::library::LibraryProjection;
    use crate::results::monitor_discovery::MonitorDiscoveryResult;
    use crate::results::workshop::AssessedWorkshopCatalogEntry;
    use crate::services::monitor_service::MonitorService;
    use lwe_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

    fn known_monitor_id() -> Option<String> {
        match MonitorService::list_monitors() {
            MonitorDiscoveryResult::Known(monitors) => {
                monitors.into_iter().next().map(|monitor| monitor.id)
            }
            MonitorDiscoveryResult::Unavailable { .. } => None,
        }
    }

    fn library_item(item_id: &str, title: &str) -> AssessedWorkshopCatalogEntry {
        AssessedWorkshopCatalogEntry {
            entry: WorkshopCatalogEntry {
                workshop_id: 7,
                title: title.to_string(),
                project_type: WorkshopProjectType::Scene,
                project_dir: std::path::PathBuf::from("/tmp/7"),
                cover_path: None,
                sync_state: WorkshopSyncState::Synced,
                supported_first_release: true,
                library_item_id: Some(item_id.to_string()),
            },
            compatibility: crate::policies::shared::compatibility_policy::compatibility_decision(
                &WorkshopCatalogEntry {
                    workshop_id: 7,
                    title: title.to_string(),
                    project_type: WorkshopProjectType::Scene,
                    project_dir: std::path::PathBuf::from("/tmp/7"),
                    cover_path: None,
                    sync_state: WorkshopSyncState::Synced,
                    supported_first_release: true,
                    library_item_id: Some(item_id.to_string()),
                },
            ),
            project_metadata: Default::default(),
        }
    }

    #[test]
    fn desktop_apply_flow_load_page_restores_known_assignments_and_reports_degraded_entries() {
        let mut assignments = BTreeMap::new();
        assignments.insert("DISPLAY-1".to_string(), "scene-7".to_string());
        assignments.insert("DISPLAY-2".to_string(), "missing-item".to_string());
        assignments.insert("DISPLAY-3".to_string(), "scene-7".to_string());

        let result = DesktopService::build_page_result(
            MonitorDiscoveryResult::Known(vec![
                crate::services::monitor_service::MonitorDescriptor {
                    id: "DISPLAY-1".to_string(),
                    name: "Primary".to_string(),
                    resolution: "1920x1080".to_string(),
                },
                crate::services::monitor_service::MonitorDescriptor {
                    id: "DISPLAY-2".to_string(),
                    name: "Secondary".to_string(),
                    resolution: "2560x1440".to_string(),
                },
            ]),
            DesktopPersistenceLoad::Loaded(assignments),
            Ok(LibraryProjection {
                entries: vec![library_item("scene-7", "Forest Scene")],
                source_catalog_count: 1,
            }),
        );

        assert!(matches!(
            result.resolved_assignments.get("DISPLAY-1"),
            Some(DesktopResolvedMonitorAssignment::Restored { item_id, item_title })
                if item_id == "scene-7" && item_title == "Forest Scene"
        ));
        assert!(matches!(
            result.resolved_assignments.get("DISPLAY-2"),
            Some(DesktopResolvedMonitorAssignment::MissingItem { item_id })
                if item_id == "missing-item"
        ));
        assert_eq!(
            result.library_item_assignments.get("scene-7"),
            Some(&vec![
                "Primary".to_string(),
                "DISPLAY-3 (missing)".to_string()
            ])
        );
        assert!(matches!(
            result.resolved_assignments.get("DISPLAY-3"),
            Some(DesktopResolvedMonitorAssignment::MissingMonitor { item_id, item_title })
                if item_id == "scene-7" && item_title.as_deref() == Some("Forest Scene")
        ));
        assert!(result.stale);
        assert_eq!(
            result.restore_issues,
            vec!["Saved assignment for missing monitor DISPLAY-3 still points to Forest Scene (scene-7).".to_string()]
        );
    }

    #[test]
    fn desktop_apply_flow_load_page_reports_library_resolution_issue_without_assignments() {
        let result = DesktopService::build_page_result(
            MonitorDiscoveryResult::Known(vec![]),
            DesktopPersistenceLoad::Loaded(BTreeMap::new()),
            Err("Library refresh failed".to_string()),
        );

        assert!(result.stale);
        assert_eq!(
            result.restore_issues,
            vec![
                "Unable to resolve desktop items against the current Library snapshot: Library refresh failed"
                    .to_string()
            ]
        );
    }

    #[test]
    fn desktop_apply_flow_load_page_keeps_assignments_when_monitor_discovery_is_unavailable() {
        let result = DesktopService::build_page_result(
            MonitorDiscoveryResult::Unavailable {
                reason: "niri unavailable".to_string(),
            },
            DesktopPersistenceLoad::Loaded(BTreeMap::from([(
                "DISPLAY-9".to_string(),
                "scene-7".to_string(),
            )])),
            Ok(LibraryProjection {
                entries: vec![library_item("scene-7", "Forest Scene")],
                source_catalog_count: 1,
            }),
        );

        assert!(!result.monitors_available);
        assert_eq!(
            result.monitor_discovery_issue.as_deref(),
            Some("niri unavailable")
        );
        assert_eq!(
            result.library_item_assignments.get("scene-7"),
            Some(&vec!["DISPLAY-9 (unavailable)".to_string()])
        );
        assert!(matches!(
            result.resolved_assignments.get("DISPLAY-9"),
            Some(DesktopResolvedMonitorAssignment::Unavailable { item_id, item_title, reason })
                if item_id == "scene-7"
                    && item_title.as_deref() == Some("Forest Scene")
                    && reason.contains("DISPLAY-9")
                    && reason.contains("niri unavailable")
        ));
        assert!(result.stale);
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
            restore_issues,
            ..
        } = result;

        assert_eq!(
            stale,
            !monitors_available || !assignments_available || !restore_issues.is_empty()
        );

        if monitors_available {
            assert!(monitor_discovery_issue.is_none());
        } else {
            assert!(monitors.is_empty());
            assert!(monitor_discovery_issue.is_some());
        }

        if assignments_available {
            assert!(persistence_issue.is_none());
        } else {
            assert!(persistence_issue.is_some());
        }
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
            DesktopApplyResult::Applied { .. } | DesktopApplyResult::PersistenceUnavailable { .. }
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
            DesktopApplyResult::Cleared { .. } | DesktopApplyResult::PersistenceUnavailable { .. }
        ));
    }
}
