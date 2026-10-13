use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use lwe_engine::{
    EngineCommand, EngineConfig, EngineEvent, EngineHandle, PlaybackSettings, WallpaperSource,
    spawn_engine,
};
use lwe_library::{WeProject, WorkshopProjectType};

use crate::results::desktop::{
    DesktopApplyResult, DesktopPageResult, DesktopResolvedMonitorAssignment,
};
use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};
use crate::results::library::LibraryProjection;
use crate::results::monitor_discovery::MonitorDiscoveryResult;
use crate::services::desktop_persistence_service::DesktopPersistenceService;
use crate::services::library_service::LibraryService;
use crate::services::monitor_service::MonitorService;
use crate::services::scene_service::SceneService;
use crate::services::wayland_capability_service::WaylandCapabilityService;

pub(crate) const LIBRARY_RESOLUTION_ISSUE_PREFIX: &str =
    "Unable to resolve desktop items against the current Library snapshot:";
const REAL_APPLY_BACKEND: &str = "lwe_engine_wayland";
const REAL_APPLY_BACKEND_TIMEOUT: Duration = Duration::from_secs(5);
const APPLY_CANCELLATION_TIMEOUT: Duration = Duration::from_secs(2);
const BACKEND_NOT_RUNNING_CLEAR_WARNING: &str =
    "Desktop runtime was not running; cleared the saved assignment only";

struct RunningDesktopApplyBackend {
    handle: EngineHandle,
    events: Receiver<EngineEvent>,
    stopped_reason: Option<String>,
    playback_settings: PlaybackSettings,
    playback_settings_issue: Option<String>,
}

static DESKTOP_APPLY_BACKEND: OnceLock<Mutex<Option<RunningDesktopApplyBackend>>> = OnceLock::new();
static STARTUP_RESTORE_ISSUES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static DESKTOP_ACTIONS: Mutex<()> = Mutex::new(());
static NEXT_APPLY_REQUEST: AtomicU64 = AtomicU64::new(1);

// Background actions and snapshots share this lock, including runtime and saved state.
fn with_desktop_action<T>(action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let _guard = DESKTOP_ACTIONS
        .lock()
        .map_err(|_| "Desktop action lock was poisoned".to_string())?;
    action()
}

fn next_apply_request_id() -> Result<u64, String> {
    let mut current = NEXT_APPLY_REQUEST.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or_else(|| "Desktop apply request IDs were exhausted".to_string())?;
        match NEXT_APPLY_REQUEST.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Ok(current),
            Err(updated) => current = updated,
        }
    }
}

fn desktop_apply_backend_slot() -> &'static Mutex<Option<RunningDesktopApplyBackend>> {
    DESKTOP_APPLY_BACKEND.get_or_init(|| Mutex::new(None))
}

fn startup_restore_issues_slot() -> &'static Mutex<Vec<String>> {
    STARTUP_RESTORE_ISSUES.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(test)]
pub(crate) fn real_desktop_flow_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static REAL_DESKTOP_FLOW_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    REAL_DESKTOP_FLOW_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("real desktop flow test lock was poisoned")
}

pub struct DesktopService;

impl DesktopService {
    pub(crate) fn playback_settings_issue(
        settings: PlaybackSettings,
    ) -> Result<Option<String>, String> {
        settings.validate()?;
        with_desktop_action(|| {
            let backend = desktop_apply_backend_slot()
                .lock()
                .map_err(|_| "Desktop apply backend lock was poisoned".to_string())?;
            let Some(backend) = backend
                .as_ref()
                .filter(|backend| backend.handle.is_running())
            else {
                return Ok(None);
            };
            let issue = backend.playback_settings_issue.clone().or_else(|| {
                (backend.playback_settings != settings).then(|| {
                    "Saved playback settings have not been applied to the running wallpaper"
                        .to_string()
                })
            });
            Ok(issue)
        })
    }

    pub(crate) fn update_playback_settings(settings: PlaybackSettings) -> Result<(), String> {
        settings.validate()?;
        with_desktop_action(|| {
            let mut backend = desktop_apply_backend_slot()
                .lock()
                .map_err(|_| "Desktop apply backend lock was poisoned".to_string())?;
            let Some(backend) = backend
                .as_mut()
                .filter(|backend| backend.handle.is_running())
            else {
                // Browse-only sessions need no runtime; the next startup reads the saved preference.
                return Ok(());
            };
            if backend.playback_settings == settings && backend.playback_settings_issue.is_none() {
                return Ok(());
            }
            let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;
            let (reply, received) = std::sync::mpsc::channel();
            let result = backend.handle.send(EngineCommand::UpdatePlaybackSettings {
                settings, deadline, reply,
            }).map_err(|error| {
                format!("Failed to send playback settings to {REAL_APPLY_BACKEND}: {error}")
            }).and_then(|()| {
                received.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .map_err(|error| match error {
                        RecvTimeoutError::Timeout => format!("Timed out waiting for {REAL_APPLY_BACKEND} to acknowledge playback settings"),
                        RecvTimeoutError::Disconnected => format!("{REAL_APPLY_BACKEND} stopped before acknowledging playback settings"),
                    })?
            });
            match result {
                Ok(()) => {
                    backend.playback_settings = settings;
                    backend.playback_settings_issue = None;
                    Ok(())
                }
                Err(reason) => {
                    backend.playback_settings_issue = Some(reason.clone());
                    Err(reason)
                }
            }
        })
    }

    pub(crate) fn library_resolution_issue(reason: &str) -> String {
        format!("{LIBRARY_RESOLUTION_ISSUE_PREFIX} {reason}")
    }

    pub fn load_page() -> Result<DesktopPageResult, String> {
        with_desktop_action(|| {
            Self::load_page_with_projection_and_monitors_unlocked(
                LibraryService::load_projection_snapshot()
                    .or_else(|_| LibraryService::load_projection()),
                MonitorService::list_monitors(),
            )
        })
    }

    pub fn restore_saved_assignments() -> Result<(), String> {
        with_desktop_action(Self::restore_saved_assignments_unlocked)
    }

    fn restore_saved_assignments_unlocked() -> Result<(), String> {
        // Read after acquiring the action lock: a preceding user action may have
        // changed saved assignments while this background restore was waiting.
        let page = Self::load_page_with_projection_and_monitors_unlocked(
            LibraryService::load_projection_snapshot()
                .or_else(|_| LibraryService::load_projection()),
            MonitorService::list_monitors_uncached(),
        )?;
        let issues = Self::restore_saved_assignments_with(&page, |monitor, item_id| {
            let request_id = Self::apply_with_real_backend(monitor, item_id)?;
            Self::finalize_real_apply(monitor, request_id);
            Ok(())
        });

        for issue in &issues {
            eprintln!("desktop restore skipped: {issue}");
        }

        Self::record_startup_restore_issues(issues);

        Ok(())
    }

    fn record_startup_restore_issues(issues: Vec<String>) {
        if let Ok(mut stored) = startup_restore_issues_slot().lock() {
            *stored = issues;
        }
    }

    fn take_startup_restore_issues() -> Vec<String> {
        startup_restore_issues_slot()
            .lock()
            .map(|mut issues| std::mem::take(&mut *issues))
            .unwrap_or_default()
    }

    pub(crate) fn load_page_with_projection(
        library_projection: Result<LibraryProjection, String>,
    ) -> Result<DesktopPageResult, String> {
        with_desktop_action(|| {
            Self::load_page_with_projection_and_monitors_unlocked(
                library_projection,
                MonitorService::list_monitors(),
            )
        })
    }

    // Every caller already owns DESKTOP_ACTIONS; restore must not reacquire it.
    fn load_page_with_projection_and_monitors_unlocked(
        library_projection: Result<LibraryProjection, String>,
        monitors: MonitorDiscoveryResult,
    ) -> Result<DesktopPageResult, String> {
        let assignments = match DesktopPersistenceService::for_user_path() {
            Ok(service) => service.load_state(),
            Err(reason) => DesktopPersistenceLoad::Unavailable { reason },
        };

        let mut result = Self::build_page_result(monitors, assignments, library_projection);

        let (running_outputs, runtime_issue) = Self::runtime_running_outputs();
        result.running_outputs = running_outputs;
        result.runtime_issue = runtime_issue;

        for issue in Self::take_startup_restore_issues() {
            if !result.restore_issues.contains(&issue) {
                result.restore_issues.push(issue);
            }
        }

        result.stale =
            result.stale || result.runtime_issue.is_some() || !result.restore_issues.is_empty();

        Ok(result)
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
            running_outputs: BTreeSet::new(),
            restore_issues,
            runtime_issue: None,
            monitors_available,
            monitor_discovery_issue,
            persistence_issue,
            assignments_available,
            stale,
        }
    }

    fn runtime_running_outputs() -> (BTreeSet<String>, Option<String>) {
        match Self::runtime_running_outputs_result() {
            Ok(status) => status,
            Err(reason) => (BTreeSet::new(), Some(reason)),
        }
    }

    fn runtime_running_outputs_result() -> Result<(BTreeSet<String>, Option<String>), String> {
        let mut backend_guard = desktop_apply_backend_slot()
            .lock()
            .map_err(|_| "Desktop apply backend lock was poisoned".to_string())?;
        let Some(backend) = backend_guard.as_mut() else {
            return Ok((BTreeSet::new(), None));
        };

        if !backend.handle.is_running() {
            return Err(Self::stopped_backend_reason(
                &backend.events,
                &mut backend.stopped_reason,
            ));
        }

        backend
            .handle
            .send(EngineCommand::GetStatus)
            .map_err(|error| format!("Failed to request desktop runtime status: {error}"))?;

        Self::wait_for_status(backend)
    }

    fn wait_for_status(
        backend: &mut RunningDesktopApplyBackend,
    ) -> Result<(BTreeSet<String>, Option<String>), String> {
        let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;

        loop {
            match Self::recv_backend_event(&backend.events, deadline)? {
                Some(EngineEvent::Status(status)) => {
                    let issue = if status.output_errors.is_empty() {
                        None
                    } else {
                        let mut errors: Vec<_> = status.output_errors.into_iter().collect();
                        errors.sort_by(|a, b| a.0.cmp(&b.0));
                        Some(
                            errors
                                .into_iter()
                                .map(|(output, reason)| format!("{output}: {reason}"))
                                .collect::<Vec<_>>()
                                .join("; "),
                        )
                    };
                    return Ok((
                        status
                            .active_wallpapers
                            .into_iter()
                            .filter_map(|(output, path)| path.map(|_| output))
                            .collect(),
                        issue,
                    ));
                }
                Some(EngineEvent::Error(reason)) => {
                    return Err(format!(
                        "{REAL_APPLY_BACKEND} could not report runtime status: {reason}"
                    ));
                }
                Some(_) => {}
                None => {
                    return Err(format!(
                        "Timed out waiting for {REAL_APPLY_BACKEND} runtime status"
                    ));
                }
            }
        }
    }

    fn stopped_backend_reason(
        events: &Receiver<EngineEvent>,
        stored: &mut Option<String>,
    ) -> String {
        if stored.is_none() {
            let reason = events.try_iter().find_map(|event| match event {
                EngineEvent::Error(reason) => Some(reason),
                EngineEvent::OutputError { output, reason } => Some(format!("{output}: {reason}")),
                _ => None,
            });
            *stored = Some(format!(
                "{REAL_APPLY_BACKEND} stopped: {}",
                reason.unwrap_or_else(|| "the runtime thread exited unexpectedly".to_string())
            ));
        }
        stored
            .clone()
            .expect("stopped runtime reason was initialized")
    }

    pub(crate) fn restore_saved_assignments_with<F>(
        page: &DesktopPageResult,
        mut apply: F,
    ) -> Vec<String>
    where
        F: FnMut(&crate::services::monitor_service::MonitorDescriptor, &str) -> Result<(), String>,
    {
        let monitor_map = page
            .monitors
            .iter()
            .map(|monitor| (monitor.id.as_str(), monitor))
            .collect::<BTreeMap<_, _>>();
        let mut issues = Vec::new();

        for (monitor_id, assignment) in &page.resolved_assignments {
            if let DesktopResolvedMonitorAssignment::Restored { item_id, .. } = assignment {
                let Some(monitor) = monitor_map.get(monitor_id.as_str()) else {
                    issues.push(format!(
                        "Saved assignment for monitor {monitor_id} was marked restorable but no active monitor descriptor was available."
                    ));
                    continue;
                };

                if let Err(reason) = apply(monitor, item_id) {
                    issues.push(format!(
                        "Failed to restore saved assignment for monitor {monitor_id} with item {item_id}: {reason}"
                    ));
                }
            }
        }

        issues
    }

    pub fn apply_to_monitor(monitor_id: &str, item_id: &str) -> Result<DesktopApplyResult, String> {
        with_desktop_action(|| Self::apply_to_monitor_unlocked(monitor_id, item_id))
    }

    fn apply_to_monitor_unlocked(
        monitor_id: &str,
        item_id: &str,
    ) -> Result<DesktopApplyResult, String> {
        Self::apply_properties_to_monitor_unlocked(monitor_id, item_id, None)
    }

    pub fn update_scene_properties(
        monitor_id: &str,
        item_id: &str,
        values: &crate::results::session_persistence::ScenePropertyValues,
    ) -> Result<DesktopApplyResult, String> {
        with_desktop_action(|| {
            if LibraryService::inspect_item(item_id)?.entry.project_type
                != WorkshopProjectType::Scene
            {
                return Err("Properties can only be applied to Scene wallpapers".into());
            }
            Self::apply_properties_to_monitor_unlocked(monitor_id, item_id, Some(values))
        })
    }

    fn apply_properties_to_monitor_unlocked(
        monitor_id: &str,
        item_id: &str,
        values: Option<&crate::results::session_persistence::ScenePropertyValues>,
    ) -> Result<DesktopApplyResult, String> {
        if let Some(result) = Self::unsupported_apply_result(item_id)? {
            return Ok(result);
        }

        let monitors = MonitorService::list_monitors_uncached();

        match MonitorService::resolve_specific_monitor(&monitors, monitor_id) {
            MonitorDiscoveryResult::Known(monitors) if monitors.is_empty() => {
                Ok(DesktopApplyResult::MonitorNotFound {
                    monitor_id: monitor_id.to_string(),
                })
            }
            MonitorDiscoveryResult::Known(monitors) => {
                let monitor = monitors
                    .into_iter()
                    .next()
                    .ok_or_else(|| format!("Monitor {monitor_id} unexpectedly resolved empty"))?;

                let request_result = match values {
                    Some(values) => Self::apply_with_real_backend_values(&monitor, item_id, values),
                    None => Self::apply_with_real_backend(&monitor, item_id),
                };
                let request_id = match request_result {
                    Ok(request_id) => request_id,
                    Err(reason) => return Ok(DesktopApplyResult::BackendUnavailable { reason }),
                };

                let persistence = match DesktopPersistenceService::for_user_path() {
                    Ok(service) => service,
                    Err(reason) => {
                        let reason =
                            Self::rollback_persistence_failure(&monitor, request_id, reason);
                        return Ok(DesktopApplyResult::PersistenceUnavailable { reason });
                    }
                };

                let write = match values {
                    Some(values) => {
                        persistence.save_assignment_with_properties(monitor_id, item_id, values)
                    }
                    None => persistence.save_assignment(monitor_id, item_id),
                };
                match write {
                    DesktopPersistenceWrite::Saved => {
                        Self::finalize_real_apply(&monitor, request_id);
                        Ok(DesktopApplyResult::AppliedWithBackend {
                            monitor_id: monitor_id.to_string(),
                            item_id: item_id.to_string(),
                            backend: REAL_APPLY_BACKEND.to_string(),
                        })
                    }
                    DesktopPersistenceWrite::Cleared => {
                        Ok(DesktopApplyResult::PersistenceUnavailable {
                            reason: Self::rollback_persistence_failure(
                                &monitor,
                                request_id,
                                "Desktop persistence returned a clear result while saving"
                                    .to_string(),
                            ),
                        })
                    }
                    DesktopPersistenceWrite::Unavailable { reason } => {
                        let reason =
                            Self::rollback_persistence_failure(&monitor, request_id, reason);
                        Ok(DesktopApplyResult::PersistenceUnavailable { reason })
                    }
                }
            }
            MonitorDiscoveryResult::Unavailable { reason } => {
                Ok(DesktopApplyResult::MonitorDiscoveryUnavailable { reason })
            }
        }
    }

    fn unsupported_apply_result(item_id: &str) -> Result<Option<DesktopApplyResult>, String> {
        let Ok(entry) = LibraryService::inspect_item(item_id) else {
            return Ok(None);
        };

        if entry.entry.project_type == WorkshopProjectType::Video {
            return Ok(None);
        }
        if entry.entry.project_type == WorkshopProjectType::Scene {
            let assessment = SceneService::assess_project(&entry.entry.project_dir);
            if assessment.runnable {
                return Ok(None);
            }
            return Ok(Some(DesktopApplyResult::UnsupportedItem {
                item_id: item_id.to_owned(),
                reason: assessment
                    .issues
                    .into_iter()
                    .map(|issue| issue.detail)
                    .collect::<Vec<_>>()
                    .join("; "),
            }));
        }

        Ok(Self::unsupported_apply_result_for_project_type(
            item_id,
            entry.entry.project_type,
        ))
    }

    fn unsupported_apply_result_for_project_type(
        item_id: &str,
        project_type: WorkshopProjectType,
    ) -> Option<DesktopApplyResult> {
        if project_type == WorkshopProjectType::Video {
            return None;
        }

        Some(DesktopApplyResult::UnsupportedItem {
            item_id: item_id.to_string(),
            reason: format!(
                "{:?} wallpapers are recognized for compatibility reporting, but the current verified runtime can only apply video wallpapers.",
                project_type
            ),
        })
    }

    fn apply_with_real_backend(
        monitor: &crate::services::monitor_service::MonitorDescriptor,
        item_id: &str,
    ) -> Result<u64, String> {
        let values = DesktopPersistenceService::load_scene_properties(&monitor.id, item_id)?;
        Self::apply_with_real_backend_values(monitor, item_id, &values)
    }

    fn apply_with_real_backend_values(
        monitor: &crate::services::monitor_service::MonitorDescriptor,
        item_id: &str,
        values: &crate::results::session_persistence::ScenePropertyValues,
    ) -> Result<u64, String> {
        Self::ensure_real_backend_supported()?;

        let source = if values.is_empty() {
            Self::resolve_real_apply_source(item_id)?
        } else {
            Self::resolve_real_apply_source_with_properties(item_id, values)?
        };
        let path = source.path().to_path_buf();
        let mut backend_guard = Self::ensure_running_apply_backend()?;
        let backend = backend_guard
            .as_mut()
            .ok_or_else(|| "Desktop apply backend failed to initialize".to_string())?;

        Self::wait_for_output(backend, &monitor.backend_output_id)?;
        let request_id = next_apply_request_id()?;
        let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;
        backend
            .handle
            .send(EngineCommand::ApplyWallpaper {
                request_id,
                deadline,
                source,
                output: Some(monitor.backend_output_id.clone()),
            })
            .map_err(|error| format!("Failed to send real desktop apply command: {error}"))?;
        Self::wait_for_apply_or_cancel(
            &backend.events,
            request_id,
            &monitor.backend_output_id,
            &path,
            deadline,
            || {
                backend
                    .handle
                    .send(EngineCommand::CancelApply {
                        request_id,
                        output: monitor.backend_output_id.clone(),
                    })
                    .map_err(|error| format!("Failed to send apply cancellation: {error}"))
            },
        )?;
        let committed = backend
            .handle
            .send(EngineCommand::CommitApply {
                request_id,
                output: monitor.backend_output_id.clone(),
            })
            .map_err(|error| format!("Failed to commit wallpaper presentation: {error}"))
            .and_then(|_| {
                Self::wait_for_commit(
                    &backend.events,
                    request_id,
                    &monitor.backend_output_id,
                    Instant::now() + REAL_APPLY_BACKEND_TIMEOUT,
                )
            });
        if let Err(reason) = committed {
            return Err(Self::cancel_backend_apply(
                backend,
                request_id,
                &monitor.backend_output_id,
            )
            .err()
            .map(|error| format!("{reason} Cancellation failed: {error}"))
            .unwrap_or(reason));
        }
        Ok(request_id)
    }

    fn wait_for_commit(
        events: &Receiver<EngineEvent>,
        request_id: u64,
        output: &str,
        deadline: Instant,
    ) -> Result<(), String> {
        loop {
            match Self::recv_backend_event(events, deadline)? {
                Some(EngineEvent::ApplyCommitted {
                    request_id: id,
                    output: name,
                    result,
                }) if id == request_id && name == output => return result,
                Some(EngineEvent::ApplyFailed {
                    request_id: id,
                    output: name,
                    reason,
                }) if id == request_id && name == output => return Err(reason),
                Some(EngineEvent::Error(reason)) => return Err(reason),
                Some(_) => {}
                None => {
                    return Err(format!(
                        "Timed out waiting for {REAL_APPLY_BACKEND} presentation commit"
                    ));
                }
            }
        }
    }

    fn cancel_backend_apply(
        backend: &RunningDesktopApplyBackend,
        request_id: u64,
        output: &str,
    ) -> Result<(), String> {
        backend
            .handle
            .send(EngineCommand::CancelApply {
                request_id,
                output: output.to_owned(),
            })
            .map_err(|error| format!("Failed to send apply cancellation: {error}"))?;
        Self::wait_for_apply_cancellation(
            &backend.events,
            request_id,
            output,
            Instant::now() + APPLY_CANCELLATION_TIMEOUT,
        )
    }

    fn rollback_persistence_failure(
        monitor: &crate::services::monitor_service::MonitorDescriptor,
        request_id: u64,
        reason: String,
    ) -> String {
        let result = desktop_apply_backend_slot()
            .lock()
            .map_err(|_| "Desktop backend lock was poisoned".to_string())
            .and_then(|backend| {
                backend
                    .as_ref()
                    .ok_or_else(|| "Desktop backend stopped before rollback".to_string())
                    .and_then(|backend| {
                        Self::cancel_backend_apply(backend, request_id, &monitor.backend_output_id)
                    })
            });
        result
            .err()
            .map(|error| format!("{reason} Runtime rollback failed: {error}"))
            .unwrap_or(reason)
    }

    fn finalize_real_apply(
        monitor: &crate::services::monitor_service::MonitorDescriptor,
        request_id: u64,
    ) {
        // Finalization frees rollback resources. Its failure cannot invalidate
        // an already confirmed presentation and successfully saved assignment.
        if let Ok(backend) = desktop_apply_backend_slot().lock()
            && let Some(backend) = backend.as_ref()
            && let Err(error) = backend.handle.send(EngineCommand::FinalizeApply {
                request_id,
                output: monitor.backend_output_id.clone(),
            })
        {
            eprintln!("desktop apply rollback cleanup deferred: {error}");
        }
    }

    fn wait_for_apply_or_cancel(
        events: &Receiver<EngineEvent>,
        request_id: u64,
        output_name: &str,
        path: &Path,
        deadline: Instant,
        cancel: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        match Self::wait_for_apply(events, request_id, output_name, path, deadline) {
            Ok(()) => Ok(()),
            Err(reason) => {
                // Cancel even a queued success ACK: it was not accepted or saved.
                // The engine only releases resources still owned by this request.
                let cancelled = cancel().and_then(|_| {
                    Self::wait_for_apply_cancellation(
                        events,
                        request_id,
                        output_name,
                        Instant::now() + APPLY_CANCELLATION_TIMEOUT,
                    )
                });
                match cancelled {
                    Ok(()) => Err(reason),
                    Err(cancellation) => {
                        Err(format!("{reason} Cancellation failed: {cancellation}"))
                    }
                }
            }
        }
    }

    fn ensure_real_backend_supported() -> Result<(), String> {
        WaylandCapabilityService::ensure_dynamic_wallpaper_supported()
            .map(|_| ())
            .map_err(|reason| {
                format!(
                    "{REAL_APPLY_BACKEND} is unavailable because required Wayland runtime capabilities are missing: {reason}"
                )
            })
    }

    fn resolve_real_apply_source(item_id: &str) -> Result<WallpaperSource, String> {
        Self::resolve_real_apply_source_with_properties(item_id, &BTreeMap::new())
    }

    fn resolve_real_apply_source_with_properties(
        item_id: &str,
        values: &crate::results::session_persistence::ScenePropertyValues,
    ) -> Result<WallpaperSource, String> {
        let entry = LibraryService::inspect_item(item_id).map_err(|reason| {
            format!("Library item {item_id} is unavailable for real desktop apply: {reason}")
        })?;

        if entry.entry.project_type == WorkshopProjectType::Scene {
            return Ok(WallpaperSource::Scene {
                path: entry.entry.project_dir.clone(),
                prepared: Arc::new(SceneService::prepare_project_with_properties(
                    &entry.entry.project_dir,
                    values,
                )?),
            });
        }
        if entry.entry.project_type != WorkshopProjectType::Video {
            return Err(format!(
                "Library item {item_id} uses {:?}, but the current real desktop apply path only supports synced video items on this machine",
                entry.entry.project_type
            ));
        }
        if !values.is_empty() {
            return Err("Properties can only be applied to Scene wallpapers".into());
        }

        let project = WeProject::load(&entry.entry.project_dir).map_err(|error| {
            format!(
                "Library item {item_id} could not load project metadata for real desktop apply: {error}"
            )
        })?;
        let path = project.main_file(&entry.entry.project_dir).ok_or_else(|| {
            format!(
                "Library item {item_id} does not expose a primary video asset for real desktop apply"
            )
        })?;

        if !path.is_file() {
            return Err(format!(
                "Library item {item_id} points to missing video asset {}",
                path.display()
            ));
        }

        Ok(WallpaperSource::Video { path })
    }

    fn ensure_running_apply_backend()
    -> Result<std::sync::MutexGuard<'static, Option<RunningDesktopApplyBackend>>, String> {
        let mut backend = desktop_apply_backend_slot()
            .lock()
            .map_err(|_| "Desktop apply backend lock was poisoned".to_string())?;
        let should_restart = backend
            .as_ref()
            .map(|backend| !backend.handle.is_running())
            .unwrap_or(true);

        if should_restart {
            *backend = Some(Self::start_apply_backend()?);
        }

        Ok(backend)
    }

    fn start_apply_backend() -> Result<RunningDesktopApplyBackend, String> {
        let settings = match crate::services::settings_persistence_service::SettingsPersistenceService::for_user_path()?.load_settings() {
            crate::results::settings_persistence::SettingsPersistenceLoad::Loaded(settings) => settings.playback_settings(),
            crate::results::settings_persistence::SettingsPersistenceLoad::Unavailable { reason } => return Err(reason),
        };
        settings.validate()?;
        let config = EngineConfig {
            fps_limit: settings.fps_limit,
            pause_on_battery: settings.pause_on_battery,
            ..EngineConfig::default()
        };
        let (handle, events) = spawn_engine(config)
            .map_err(|error| format!("Failed to start {REAL_APPLY_BACKEND}: {error}"))?;
        let mut backend = RunningDesktopApplyBackend {
            handle,
            events,
            stopped_reason: None,
            playback_settings: settings,
            playback_settings_issue: None,
        };

        Self::wait_for_backend_start(&mut backend)?;

        Ok(backend)
    }

    fn wait_for_backend_start(backend: &mut RunningDesktopApplyBackend) -> Result<(), String> {
        let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;

        loop {
            match Self::recv_backend_event(&backend.events, deadline)? {
                Some(EngineEvent::Started) => return Ok(()),
                Some(EngineEvent::Error(reason)) => {
                    return Err(format!("{REAL_APPLY_BACKEND} failed to start: {reason}"));
                }
                Some(_) => {}
                None => {
                    return Err(format!(
                        "Timed out waiting for {REAL_APPLY_BACKEND} to start"
                    ));
                }
            }
        }
    }

    fn wait_for_output(
        backend: &mut RunningDesktopApplyBackend,
        output_name: &str,
    ) -> Result<(), String> {
        let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;
        backend
            .handle
            .send(EngineCommand::GetOutputs)
            .map_err(|error| {
                format!("Failed to request outputs from {REAL_APPLY_BACKEND}: {error}")
            })?;

        loop {
            match Self::recv_backend_event(&backend.events, deadline)? {
                Some(EngineEvent::OutputAdded(info)) => {
                    if Self::output_added_matches(&info, output_name) {
                        return Ok(());
                    }
                }
                Some(EngineEvent::OutputsList(outputs)) => {
                    Self::ensure_output_list_contains(&outputs, output_name)?;
                    return Ok(());
                }
                Some(EngineEvent::Error(reason)) => {
                    return Err(format!(
                        "{REAL_APPLY_BACKEND} could not prepare output {output_name}: {reason}"
                    ));
                }
                Some(_) => {}
                None => {
                    return Err(format!(
                        "Timed out waiting for output {output_name} in {REAL_APPLY_BACKEND}"
                    ));
                }
            }
        }
    }

    fn output_added_matches(output: &lwe_engine::OutputInfo, output_name: &str) -> bool {
        output.name == output_name
    }

    fn ensure_output_list_contains(
        outputs: &[lwe_engine::OutputInfo],
        output_name: &str,
    ) -> Result<(), String> {
        if outputs.iter().any(|output| output.name == output_name) {
            return Ok(());
        }

        let available = outputs
            .iter()
            .map(|output| output.name.as_str())
            .collect::<Vec<_>>();

        if available.is_empty() {
            Err(format!(
                "{REAL_APPLY_BACKEND} did not report any active outputs while preparing {output_name}. Confirm the Wayland session exposes outputs to the runtime backend."
            ))
        } else {
            Err(format!(
                "{REAL_APPLY_BACKEND} reported outputs [{}] but not requested output {output_name}. Reselect the monitor in LWE and check for a compositor output-name mismatch.",
                available.join(", ")
            ))
        }
    }

    fn wait_for_apply(
        events: &Receiver<EngineEvent>,
        request_id: u64,
        output_name: &str,
        path: &Path,
        deadline: Instant,
    ) -> Result<(), String> {
        loop {
            match Self::recv_backend_event(events, deadline)? {
                Some(EngineEvent::WallpaperApplied {
                    request_id: applied_request,
                    output,
                    path: applied_path,
                }) if applied_request == request_id
                    && output == output_name
                    && applied_path == path =>
                {
                    return Ok(());
                }
                Some(EngineEvent::ApplyFailed {
                    request_id: failed_request,
                    output,
                    reason,
                }) if failed_request == request_id && output == output_name => {
                    return Err(format!(
                        "{REAL_APPLY_BACKEND} failed to apply {} to {output_name}: {reason}",
                        path.display()
                    ));
                }
                Some(EngineEvent::Error(reason)) => {
                    return Err(format!(
                        "{REAL_APPLY_BACKEND} failed to apply {} to {output_name}: {reason}",
                        path.display()
                    ));
                }
                Some(_) => {}
                None => {
                    return Err(Self::apply_timeout_message(output_name, path));
                }
            }
        }
    }

    fn wait_for_apply_cancellation(
        events: &Receiver<EngineEvent>,
        request_id: u64,
        output_name: &str,
        deadline: Instant,
    ) -> Result<(), String> {
        loop {
            match Self::recv_backend_event(events, deadline)? {
                Some(EngineEvent::ApplyCancelled {
                    request_id: cancelled_request,
                    output,
                    result,
                }) if cancelled_request == request_id && output == output_name => return result,
                Some(EngineEvent::Error(reason)) => return Err(reason),
                Some(_) => {}
                None => {
                    return Err(format!(
                        "Timed out cancelling request {request_id} on {output_name}"
                    ));
                }
            }
        }
    }

    fn apply_timeout_message(output_name: &str, path: &Path) -> String {
        format!(
            "Timed out waiting for {REAL_APPLY_BACKEND} to apply {} to {output_name}. Check Wayland layer-shell, EGL, mpv, and video asset availability.",
            path.display()
        )
    }

    fn recv_backend_event(
        events: &Receiver<EngineEvent>,
        deadline: Instant,
    ) -> Result<Option<EngineEvent>, String> {
        let now = Instant::now();
        if now >= deadline {
            return Ok(None);
        }

        match events.recv_timeout(deadline.saturating_duration_since(now)) {
            Ok(event) if Instant::now() < deadline => Ok(Some(event)),
            Ok(_) => Ok(None),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(format!(
                "{REAL_APPLY_BACKEND} stopped before completing the desktop apply request"
            )),
        }
    }

    pub fn clear_monitor(monitor_id: &str) -> Result<DesktopApplyResult, String> {
        with_desktop_action(|| Self::clear_monitor_unlocked(monitor_id))
    }

    fn clear_monitor_unlocked(monitor_id: &str) -> Result<DesktopApplyResult, String> {
        let monitors = MonitorService::list_monitors_uncached();

        match MonitorService::resolve_specific_monitor(&monitors, monitor_id) {
            MonitorDiscoveryResult::Known(monitors) if monitors.is_empty() => {
                Ok(DesktopApplyResult::MonitorNotFound {
                    monitor_id: monitor_id.to_string(),
                })
            }
            MonitorDiscoveryResult::Known(monitors) => {
                let monitor = monitors
                    .into_iter()
                    .next()
                    .ok_or_else(|| format!("Monitor {monitor_id} unexpectedly resolved empty"))?;

                let runtime_warning = Self::clear_with_real_backend(&monitor).err();

                let persistence = match DesktopPersistenceService::for_user_path() {
                    Ok(service) => service,
                    Err(reason) => {
                        return Ok(DesktopApplyResult::PersistenceUnavailable { reason });
                    }
                };

                match persistence.clear_assignment(monitor_id) {
                    DesktopPersistenceWrite::Cleared => match runtime_warning {
                        Some(warning) => Ok(DesktopApplyResult::ClearedWithBackendWarning {
                            monitor_id: monitor_id.to_string(),
                            warning,
                        }),
                        None => Ok(DesktopApplyResult::Cleared {
                            monitor_id: monitor_id.to_string(),
                        }),
                    },
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

    fn clear_with_real_backend(
        monitor: &crate::services::monitor_service::MonitorDescriptor,
    ) -> Result<(), String> {
        let mut backend_guard = desktop_apply_backend_slot()
            .lock()
            .map_err(|_| "Desktop apply backend lock was poisoned".to_string())?;

        let Some(backend) = backend_guard.as_mut() else {
            return Err(BACKEND_NOT_RUNNING_CLEAR_WARNING.to_string());
        };

        if !backend.handle.is_running() {
            *backend_guard = None;
            return Err(BACKEND_NOT_RUNNING_CLEAR_WARNING.to_string());
        }

        Self::wait_for_output(backend, &monitor.backend_output_id)?;
        backend
            .handle
            .send(EngineCommand::ClearWallpaper {
                output: Some(monitor.backend_output_id.clone()),
            })
            .map_err(|error| format!("Failed to send real desktop clear command: {error}"))?;

        Self::wait_for_clear(backend, &monitor.backend_output_id)
    }

    fn wait_for_clear(
        backend: &mut RunningDesktopApplyBackend,
        output_name: &str,
    ) -> Result<(), String> {
        let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;

        loop {
            match Self::recv_backend_event(&backend.events, deadline)? {
                Some(EngineEvent::WallpaperCleared { output }) if output == output_name => {
                    return Ok(());
                }
                Some(EngineEvent::OutputError { output, reason }) if output == output_name => {
                    return Err(format!(
                        "{REAL_APPLY_BACKEND} failed to clear output {output_name}: {reason}"
                    ));
                }
                Some(EngineEvent::Error(reason)) => {
                    return Err(format!(
                        "{REAL_APPLY_BACKEND} failed to clear output {output_name}: {reason}"
                    ));
                }
                Some(_) => {}
                None => {
                    return Err(format!(
                        "Timed out waiting for {REAL_APPLY_BACKEND} to clear output {output_name}"
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "desktop_reliability_tests.rs"]
mod reliability_tests;

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::results::desktop::DesktopApplyResult;
    use crate::results::desktop::DesktopResolvedMonitorAssignment;
    use crate::results::library::LibraryProjection;
    use crate::results::monitor_discovery::MonitorDiscoveryResult;
    use crate::results::workshop::AssessedWorkshopCatalogEntry;
    use crate::services::library_service::LibraryService;
    use crate::services::monitor_service::MonitorService;
    use lwe_engine::OutputInfo;
    use lwe_library::WeProject;
    use lwe_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

    fn applied(request_id: u64, output: &str) -> EngineEvent {
        EngineEvent::WallpaperApplied {
            request_id,
            output: output.to_string(),
            path: PathBuf::from("/tmp/video.mp4"),
        }
    }

    #[test]
    fn desktop_apply_flow_same_path_retry_ignores_late_ack() {
        let (tx, events) = std::sync::mpsc::channel();
        tx.send(applied(10, "eDP-1")).unwrap();
        tx.send(applied(11, "eDP-1")).unwrap();

        DesktopService::wait_for_apply(
            &events,
            11,
            "eDP-1",
            Path::new("/tmp/video.mp4"),
            Instant::now() + Duration::from_secs(1),
        )
        .unwrap();
        assert!(matches!(
            events.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ));
    }

    #[test]
    fn desktop_apply_flow_failure_is_scoped_to_request_and_output() {
        let (tx, events) = std::sync::mpsc::channel();
        for (request_id, output) in [(10, "eDP-1"), (11, "HDMI-A-1")] {
            tx.send(EngineEvent::ApplyFailed {
                request_id,
                output: output.to_string(),
                reason: "unrelated application".to_string(),
            })
            .unwrap();
        }
        tx.send(EngineEvent::OutputError {
            output: "HDMI-A-1".to_string(),
            reason: "unrelated clear".to_string(),
        })
        .unwrap();
        tx.send(EngineEvent::ApplyFailed {
            request_id: 11,
            output: "eDP-1".to_string(),
            reason: "this application failed".to_string(),
        })
        .unwrap();

        let reason = DesktopService::wait_for_apply(
            &events,
            11,
            "eDP-1",
            Path::new("/tmp/video.mp4"),
            Instant::now() + Duration::from_secs(1),
        )
        .unwrap_err();
        assert!(reason.contains("this application failed"));
        assert!(!reason.contains("unrelated"));
    }

    #[test]
    fn desktop_apply_flow_expired_ack_is_not_accepted() {
        let (tx, events) = std::sync::mpsc::channel();
        tx.send(applied(11, "eDP-1")).unwrap();

        let reason = DesktopService::wait_for_apply(
            &events,
            11,
            "eDP-1",
            Path::new("/tmp/video.mp4"),
            Instant::now(),
        )
        .unwrap_err();
        assert!(reason.contains("Timed out"));
        assert!(matches!(
            events.try_recv(),
            Ok(EngineEvent::WallpaperApplied { request_id: 11, .. })
        ));
    }

    #[test]
    fn desktop_apply_flow_timeout_cancels_before_same_path_retry() {
        let (tx, events) = std::sync::mpsc::channel();
        let mut cancelled = false;
        let reason = DesktopService::wait_for_apply_or_cancel(
            &events,
            11,
            "eDP-1",
            Path::new("/tmp/video.mp4"),
            Instant::now(),
            || {
                cancelled = true;
                // The timed-out first frame can have completed while cancellation
                // was in flight; only the cancellation confirmation releases the action.
                tx.send(applied(11, "eDP-1")).unwrap();
                tx.send(EngineEvent::ApplyCancelled {
                    request_id: 11,
                    output: "eDP-1".to_string(),
                    result: Ok(()),
                })
                .unwrap();
                Ok(())
            },
        )
        .unwrap_err();
        assert!(cancelled);
        assert!(reason.contains("Timed out"));
        tx.send(applied(12, "eDP-1")).unwrap();
        DesktopService::wait_for_apply(
            &events,
            12,
            "eDP-1",
            Path::new("/tmp/video.mp4"),
            Instant::now() + Duration::from_secs(1),
        )
        .unwrap();
        assert!(matches!(
            events.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ));
    }

    #[test]
    fn desktop_apply_flow_accepted_ack_does_not_cancel() {
        let (tx, events) = std::sync::mpsc::channel();
        tx.send(applied(11, "eDP-1")).unwrap();
        DesktopService::wait_for_apply_or_cancel(
            &events,
            11,
            "eDP-1",
            Path::new("/tmp/video.mp4"),
            Instant::now() + Duration::from_secs(1),
            || panic!("an accepted application must not be cancelled"),
        )
        .unwrap();
    }

    #[test]
    fn desktop_apply_flow_cancellation_waits_past_late_ack_for_own_confirmation() {
        let (tx, events) = std::sync::mpsc::channel();
        tx.send(applied(11, "eDP-1")).unwrap();
        for (request_id, output) in [(10, "eDP-1"), (11, "HDMI-A-1"), (11, "eDP-1")] {
            tx.send(EngineEvent::ApplyCancelled {
                request_id,
                output: output.to_string(),
                result: Ok(()),
            })
            .unwrap();
        }
        DesktopService::wait_for_apply_cancellation(
            &events,
            11,
            "eDP-1",
            Instant::now() + Duration::from_secs(1),
        )
        .unwrap();
        assert!(matches!(
            events.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ));
    }

    #[test]
    fn desktop_apply_flow_cancellation_reports_resource_release_failure() {
        let (tx, events) = std::sync::mpsc::channel();
        tx.send(EngineEvent::ApplyCancelled {
            request_id: 11,
            output: "eDP-1".to_string(),
            result: Err("EGL resource release failed".to_string()),
        })
        .unwrap();
        let reason = DesktopService::wait_for_apply_cancellation(
            &events,
            11,
            "eDP-1",
            Instant::now() + Duration::from_secs(1),
        )
        .unwrap_err();
        assert_eq!(reason, "EGL resource release failed");
    }

    #[test]
    fn desktop_apply_flow_action_lock_covers_runtime_and_persistence() {
        use std::sync::Arc;
        use std::sync::mpsc::channel;

        let log = Arc::new(Mutex::new(Vec::new()));
        let (entered_tx, entered_rx) = channel();
        let (resume_tx, resume_rx) = channel();
        let apply_log = log.clone();
        let apply = std::thread::spawn(move || {
            with_desktop_action(|| {
                apply_log.lock().unwrap().push("apply runtime");
                entered_tx.send(()).unwrap();
                resume_rx.recv().unwrap();
                apply_log.lock().unwrap().push("apply persistence");
                Ok(())
            })
            .unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(30)).unwrap();
        let (clear_started_tx, clear_started_rx) = channel();
        let clear_log = log.clone();
        let clear = std::thread::spawn(move || {
            clear_started_tx.send(()).unwrap();
            with_desktop_action(|| {
                clear_log.lock().unwrap().push("clear runtime");
                clear_log.lock().unwrap().push("clear persistence");
                Ok(())
            })
            .unwrap();
        });
        clear_started_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        resume_tx.send(()).unwrap();
        apply.join().unwrap();
        clear.join().unwrap();

        assert_eq!(
            *log.lock().unwrap(),
            [
                "apply runtime",
                "apply persistence",
                "clear runtime",
                "clear persistence",
            ]
        );
    }

    #[test]
    fn desktop_apply_flow_snapshot_waits_for_in_flight_assignment_save() {
        let _guard = real_desktop_flow_test_guard();
        let config_root = test_config_root();
        let _env = EnvGuard::set_config_root(&config_root);
        let persistence = DesktopPersistenceService::for_user_path().unwrap();

        for supplied_projection in [false, true] {
            assert!(matches!(
                persistence.save_assignment("DISPLAY-1", "old-item"),
                DesktopPersistenceWrite::Saved
            ));
            let action_guard = DESKTOP_ACTIONS.lock().unwrap();
            // Simulate the apply window after runtime accepted new-item but before
            // persistence caught up. Both public snapshot entrances must wait.
            let (started_tx, started_rx) = std::sync::mpsc::channel();
            let (snapshot_tx, snapshot_rx) = std::sync::mpsc::channel();
            let snapshot = std::thread::spawn(move || {
                started_tx.send(()).unwrap();
                let result = if supplied_projection {
                    DesktopService::load_page_with_projection(Ok(LibraryProjection {
                        entries: vec![library_item("new-item", "New wallpaper")],
                        source_catalog_count: 1,
                        served_from_snapshot: false,
                    }))
                } else {
                    DesktopService::load_page()
                };
                snapshot_tx.send(result).unwrap();
            });
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            let premature = snapshot_rx.recv_timeout(Duration::from_millis(30));
            assert!(matches!(premature, Err(RecvTimeoutError::Timeout)));
            assert!(matches!(
                persistence.save_assignment("DISPLAY-1", "new-item"),
                DesktopPersistenceWrite::Saved
            ));
            drop(action_guard);

            let page = snapshot_rx
                .recv_timeout(Duration::from_secs(30))
                .unwrap()
                .unwrap();
            snapshot.join().unwrap();
            assert_eq!(
                page.assignments.get("DISPLAY-1").map(String::as_str),
                Some("new-item")
            );
        }
        fs::remove_dir_all(config_root).unwrap();
    }

    struct EnvGuard {
        xdg_config_home: Option<OsString>,
        home: Option<OsString>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl EnvGuard {
        fn set_desktop_config_root(config_root: &Path) -> Self {
            let lock = crate::test_env::env_lock();
            let xdg_config_home = std::env::var_os("XDG_CONFIG_HOME");
            let home = std::env::var_os("HOME");
            assert!(home.is_some(), "Real desktop discovery requires HOME");
            unsafe {
                std::env::set_var("XDG_CONFIG_HOME", config_root);
            }
            Self {
                xdg_config_home,
                home,
                _lock: lock,
            }
        }

        fn set_config_root(config_root: &Path) -> Self {
            let lock = crate::test_env::env_lock();
            let xdg_config_home = std::env::var_os("XDG_CONFIG_HOME");
            let home = std::env::var_os("HOME");

            unsafe {
                std::env::set_var("XDG_CONFIG_HOME", config_root);
                std::env::set_var("HOME", config_root);
            }

            Self {
                xdg_config_home,
                home,
                _lock: lock,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.xdg_config_home {
                Some(value) => unsafe { std::env::set_var("XDG_CONFIG_HOME", value) },
                None => unsafe { std::env::remove_var("XDG_CONFIG_HOME") },
            }

            match &self.home {
                Some(value) => unsafe { std::env::set_var("HOME", value) },
                None => unsafe { std::env::remove_var("HOME") },
            }
        }
    }

    fn test_config_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("desktop-service-config-{unique}"))
    }

    fn known_monitor_id() -> Option<String> {
        match MonitorService::list_monitors() {
            MonitorDiscoveryResult::Known(monitors) => {
                monitors.into_iter().next().map(|monitor| monitor.id)
            }
            MonitorDiscoveryResult::Unavailable { .. } => None,
        }
    }

    fn known_monitor() -> Option<crate::services::monitor_service::MonitorDescriptor> {
        match MonitorService::list_monitors() {
            MonitorDiscoveryResult::Known(monitors) => monitors.into_iter().next(),
            MonitorDiscoveryResult::Unavailable { .. } => None,
        }
    }

    fn real_supported_library_item_id() -> Option<String> {
        let projection = LibraryService::load_projection_snapshot()
            .or_else(|_| LibraryService::load_projection())
            .ok()?;

        projection.entries.into_iter().find_map(|entry| {
            if entry.entry.project_type != WorkshopProjectType::Video {
                return None;
            }

            let project = WeProject::load(&entry.entry.project_dir).ok()?;
            let main_file = project.main_file(&entry.entry.project_dir)?;

            if !main_file.is_file() {
                return None;
            }

            entry.entry.library_item_id
        })
    }

    fn real_desktop_tests_enabled() -> bool {
        std::env::var("LWE_REAL_DESKTOP_TESTS")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false)
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
                    backend_output_id: "DISPLAY-1".to_string(),
                    name: "Primary".to_string(),
                    resolution: "1920x1080".to_string(),
                },
                crate::services::monitor_service::MonitorDescriptor {
                    id: "DISPLAY-2".to_string(),
                    backend_output_id: "DISPLAY-2".to_string(),
                    name: "Secondary".to_string(),
                    resolution: "2560x1440".to_string(),
                },
            ]),
            DesktopPersistenceLoad::Loaded(assignments),
            Ok(LibraryProjection {
                entries: vec![library_item("scene-7", "Forest Scene")],
                source_catalog_count: 1,
                served_from_snapshot: false,
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
                served_from_snapshot: false,
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
    fn desktop_apply_flow_load_page_prefers_unified_toml_session_state_over_legacy_json() {
        let _guard = real_desktop_flow_test_guard();
        let config_root = test_config_root();
        fs::create_dir_all(config_root.join("lwe")).unwrap();
        fs::create_dir_all(config_root.join("wayvid")).unwrap();
        fs::write(
            config_root.join("lwe").join("session.toml"),
            "[assignments]\nDISPLAY-1 = \"scene-from-session\"\n",
        )
        .unwrap();
        fs::write(
            config_root.join("wayvid").join("desktop-state.json"),
            "{\"assignments\":{\"DISPLAY-1\":\"scene-from-legacy\"}}",
        )
        .unwrap();
        let _env = EnvGuard::set_config_root(&config_root);

        let result = DesktopService::load_page().unwrap();

        assert_eq!(
            result.assignments.get("DISPLAY-1").map(String::as_str),
            Some("scene-from-session")
        );
    }

    #[test]
    fn desktop_apply_flow_startup_restore_applies_only_restorable_assignments() {
        let page = DesktopPageResult {
            monitors: vec![crate::services::monitor_service::MonitorDescriptor {
                id: "DISPLAY-1".to_string(),
                backend_output_id: "DISPLAY-1".to_string(),
                name: "Primary".to_string(),
                resolution: "1920x1080".to_string(),
            }],
            assignments: BTreeMap::from([
                ("DISPLAY-1".to_string(), "scene-7".to_string()),
                ("DISPLAY-2".to_string(), "missing-item".to_string()),
            ]),
            resolved_assignments: BTreeMap::from([
                (
                    "DISPLAY-1".to_string(),
                    DesktopResolvedMonitorAssignment::Restored {
                        item_id: "scene-7".to_string(),
                        item_title: "Forest Scene".to_string(),
                    },
                ),
                (
                    "DISPLAY-2".to_string(),
                    DesktopResolvedMonitorAssignment::MissingItem {
                        item_id: "missing-item".to_string(),
                    },
                ),
            ]),
            library_item_assignments: BTreeMap::new(),
            running_outputs: Default::default(),
            restore_issues: Vec::new(),
            runtime_issue: None,
            monitors_available: true,
            monitor_discovery_issue: None,
            persistence_issue: None,
            assignments_available: true,
            stale: false,
        };
        let mut applied = Vec::new();

        DesktopService::restore_saved_assignments_with(&page, |monitor, item_id| {
            applied.push((monitor.id.clone(), item_id.to_string()));
            Ok(())
        });

        assert_eq!(
            applied,
            vec![("DISPLAY-1".to_string(), "scene-7".to_string())]
        );
    }

    #[test]
    fn desktop_apply_flow_rejects_non_video_before_backend_resolution() {
        let result = DesktopService::unsupported_apply_result_for_project_type(
            "web-7",
            WorkshopProjectType::Web,
        );

        assert!(matches!(
            result,
            Some(DesktopApplyResult::UnsupportedItem { item_id, reason })
                if item_id == "web-7"
                    && reason.contains("Web wallpapers")
                    && reason.contains("only apply video wallpapers")
        ));
    }

    #[test]
    fn desktop_apply_flow_allows_video_to_continue_to_backend_resolution() {
        let result = DesktopService::unsupported_apply_result_for_project_type(
            "video-7",
            WorkshopProjectType::Video,
        );

        assert!(result.is_none());
    }

    #[test]
    fn desktop_apply_flow_backend_capability_error_names_wayland_requirement() {
        let reason = format!(
            "{REAL_APPLY_BACKEND} is unavailable because required Wayland runtime capabilities are missing: Wayland compositor does not expose zwlr_layer_shell_v1, which LWE needs for dynamic wallpaper surfaces"
        );

        assert!(reason.contains(REAL_APPLY_BACKEND));
        assert!(reason.contains("required Wayland runtime capabilities"));
        assert!(reason.contains("zwlr_layer_shell_v1"));
    }

    #[test]
    fn desktop_apply_flow_startup_restore_issues_are_consumed_after_surface() {
        DesktopService::record_startup_restore_issues(vec![
            "Failed to restore saved assignment for monitor DISPLAY-1 with item scene-7: backend unavailable".to_string(),
        ]);

        let issues = DesktopService::take_startup_restore_issues();
        let second_read = DesktopService::take_startup_restore_issues();

        assert_eq!(
            issues,
            vec!["Failed to restore saved assignment for monitor DISPLAY-1 with item scene-7: backend unavailable".to_string()]
        );
        assert!(second_read.is_empty());
    }

    #[test]
    fn desktop_apply_flow_load_page_marks_runtime_issue_as_stale_and_deduplicates_restore_issues() {
        DesktopService::record_startup_restore_issues(vec![
            "restore warning".to_string(),
            "restore warning".to_string(),
        ]);

        let mut result = DesktopService::build_page_result(
            MonitorDiscoveryResult::Known(vec![
                crate::services::monitor_service::MonitorDescriptor {
                    id: "DISPLAY-1".to_string(),
                    backend_output_id: "eDP-1".to_string(),
                    name: "Primary".to_string(),
                    resolution: "1920x1080".to_string(),
                },
            ]),
            DesktopPersistenceLoad::Loaded(BTreeMap::new()),
            Ok(LibraryProjection {
                entries: vec![],
                source_catalog_count: 0,
                served_from_snapshot: false,
            }),
        );
        result.runtime_issue =
            Some("Timed out waiting for lwe_engine_wayland runtime status".to_string());

        for issue in DesktopService::take_startup_restore_issues() {
            if !result.restore_issues.contains(&issue) {
                result.restore_issues.push(issue);
            }
        }
        result.stale =
            result.stale || result.runtime_issue.is_some() || !result.restore_issues.is_empty();

        assert!(result.stale);
        assert_eq!(
            result.runtime_issue.as_deref(),
            Some("Timed out waiting for lwe_engine_wayland runtime status")
        );
        assert_eq!(result.restore_issues, vec!["restore warning".to_string()]);
    }

    #[test]
    fn desktop_apply_flow_output_list_reports_actionable_mismatch() {
        let result = DesktopService::ensure_output_list_contains(
            &[OutputInfo {
                name: "HDMI-A-1".to_string(),
                width: 1920,
                height: 1080,
                scale: 1.0,
                position: (0, 0),
                active: true,
                hdr_capabilities: Default::default(),
            }],
            "eDP-1",
        );

        assert!(matches!(result, Err(reason) if reason.contains("HDMI-A-1")
            && reason.contains("eDP-1")
            && reason.contains("output-name mismatch")));
    }

    #[test]
    fn desktop_apply_flow_empty_output_list_reports_actionable_runtime_issue() {
        let result = DesktopService::ensure_output_list_contains(&[], "eDP-1");

        assert!(
            matches!(result, Err(reason) if reason.contains("did not report any active outputs")
            && reason.contains("Wayland session"))
        );
    }

    #[test]
    fn desktop_apply_flow_apply_timeout_mentions_runtime_layers() {
        let message = DesktopService::apply_timeout_message("eDP-1", Path::new("/tmp/video.mp4"));

        assert!(message.contains("Timed out"));
        assert!(message.contains("Wayland layer-shell"));
        assert!(message.contains("EGL"));
        assert!(message.contains("mpv"));
        assert!(message.contains("video asset"));
    }

    #[test]
    fn desktop_apply_flow_apply_to_monitor_reflects_current_monitor_discovery_state() {
        let _guard = real_desktop_flow_test_guard();
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
            DesktopApplyResult::Applied { .. }
                | DesktopApplyResult::AppliedWithBackend { .. }
                | DesktopApplyResult::BackendUnavailable { .. }
                | DesktopApplyResult::PersistenceUnavailable { .. }
        ));
    }

    #[test]
    fn desktop_apply_flow_apply_to_monitor_avoids_placeholder_result_for_real_monitor_and_library_item()
     {
        if !real_desktop_tests_enabled() {
            eprintln!("skipping real desktop apply test; set LWE_REAL_DESKTOP_TESTS=1 to enable");
            return;
        }

        let monitor = known_monitor().expect("expected a real discovered monitor on this machine");
        let item_id = real_supported_library_item_id()
            .expect("expected one real supported video Library item on this machine");

        let result = DesktopService::apply_to_monitor(&monitor.id, &item_id).unwrap();

        assert!(
            !matches!(
                result,
                DesktopApplyResult::Applied { .. }
                    | DesktopApplyResult::PersistenceUnavailable { .. }
            ),
            "expected a real backend apply result for {} and {}, got {:?}",
            monitor.id,
            item_id,
            result
        );

        let clear_result = DesktopService::clear_monitor(&monitor.id).unwrap();
        assert!(matches!(
            clear_result,
            DesktopApplyResult::Cleared { .. }
                | DesktopApplyResult::ClearedWithBackendWarning { .. }
        ));
    }

    #[test]
    fn desktop_apply_flow_clear_monitor_reflects_current_monitor_discovery_state() {
        if !real_desktop_tests_enabled() {
            eprintln!("skipping real desktop clear test; set LWE_REAL_DESKTOP_TESTS=1 to enable");
            return;
        }

        let _guard = real_desktop_flow_test_guard();

        let Some(monitor_id) = known_monitor_id() else {
            let result = DesktopService::clear_monitor("DISPLAY-1").unwrap();

            assert!(matches!(
                result,
                DesktopApplyResult::MonitorDiscoveryUnavailable { .. }
            ));
            return;
        };

        let item_id = real_supported_library_item_id()
            .expect("expected one real supported video Library item on this machine");

        let apply_result = DesktopService::apply_to_monitor(&monitor_id, &item_id).unwrap();
        assert!(
            matches!(apply_result, DesktopApplyResult::AppliedWithBackend { .. }),
            "expected real backend apply before clear, got {apply_result:?}"
        );

        let result = DesktopService::clear_monitor(&monitor_id).unwrap();

        assert!(matches!(result, DesktopApplyResult::Cleared { .. }));
    }

    #[test]
    fn desktop_apply_flow_reapplies_video_after_clear_on_same_backend() {
        if !real_desktop_tests_enabled() {
            eprintln!("skipping real desktop reapply test; set LWE_REAL_DESKTOP_TESTS=1 to enable");
            return;
        }

        let _guard = real_desktop_flow_test_guard();
        let monitor_id = known_monitor_id().expect("expected a real discovered monitor");
        let item_id = real_supported_library_item_id()
            .expect("expected one real supported video Library item");
        let mut backend_identity = None;

        for cycle in 1..=3 {
            let apply_result = DesktopService::apply_to_monitor(&monitor_id, &item_id).unwrap();
            assert!(
                matches!(apply_result, DesktopApplyResult::AppliedWithBackend { .. }),
                "cycle {cycle}: expected real backend apply, got {apply_result:?}"
            );

            let clear_result = DesktopService::clear_monitor(&monitor_id).unwrap();
            assert!(
                matches!(clear_result, DesktopApplyResult::Cleared { .. }),
                "cycle {cycle}: expected real backend clear, got {clear_result:?}"
            );

            let backend_guard = desktop_apply_backend_slot().lock().unwrap();
            let backend = backend_guard.as_ref().expect("expected a retained backend");
            assert!(
                backend.handle.is_running(),
                "cycle {cycle}: backend stopped"
            );
            let identity = backend.handle.shutdown_flag();
            if let Some(expected) = &backend_identity {
                assert!(
                    std::sync::Arc::ptr_eq(expected, &identity),
                    "cycle {cycle}: backend restarted instead of reusing resources"
                );
            } else {
                backend_identity = Some(identity);
            }
        }
    }

    #[test]
    fn commit_confirmation_matches_generation_and_rejects_expired_ack() {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(EngineEvent::ApplyCommitted {
            request_id: 7,
            output: "eDP-1".to_string(),
            result: Ok(()),
        })
        .unwrap();
        tx.send(EngineEvent::ApplyCommitted {
            request_id: 8,
            output: "eDP-1".to_string(),
            result: Ok(()),
        })
        .unwrap();
        DesktopService::wait_for_commit(&rx, 8, "eDP-1", Instant::now() + Duration::from_secs(1))
            .unwrap();
        tx.send(EngineEvent::ApplyCommitted {
            request_id: 9,
            output: "eDP-1".to_string(),
            result: Ok(()),
        })
        .unwrap();
        assert!(DesktopService::wait_for_commit(&rx, 9, "eDP-1", Instant::now()).is_err());
    }

    #[test]
    fn runtime_failure_is_retained_after_its_event_is_consumed() {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(EngineEvent::Error("Wayland connection lost".to_string()))
            .unwrap();
        tx.send(EngineEvent::Stopped).unwrap();
        let mut stored = None;
        let first = DesktopService::stopped_backend_reason(&rx, &mut stored);
        assert!(first.contains("Wayland connection lost"));
        assert_eq!(
            DesktopService::stopped_backend_reason(&rx, &mut stored),
            first
        );
    }

    #[test]
    fn desktop_settings_live_updates_preserve_assignments_and_backend() {
        if !real_desktop_tests_enabled() {
            eprintln!(
                "skipping real Settings desktop test; set LWE_REAL_DESKTOP_TESTS=1 to enable"
            );
            return;
        }
        let _desktop_guard = real_desktop_flow_test_guard();
        assert!(
            desktop_apply_backend_slot().lock().unwrap().is_none(),
            "Settings test requires its own fresh backend"
        );
        let isolated = tempfile::tempdir().unwrap();
        let _env_guard = EnvGuard::set_desktop_config_root(isolated.path());
        let monitor = known_monitor().expect("expected a real monitor");
        struct OwnedRuntimeCleanup(String);
        impl Drop for OwnedRuntimeCleanup {
            fn drop(&mut self) {
                let _ = DesktopService::clear_monitor(&self.0);
                if let Some(backend) = desktop_apply_backend_slot()
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .take()
                {
                    backend.handle.request_shutdown();
                    let _ = backend.handle.join();
                }
            }
        }
        let _cleanup = OwnedRuntimeCleanup(monitor.id.clone());
        let projection = LibraryService::load_projection().expect("expected local Library");
        let item = projection
            .entries
            .into_iter()
            .find(|entry| {
                entry
                    .entry
                    .project_dir
                    .file_name()
                    .is_some_and(|name| name == "3337606455")
                    && entry
                        .project_metadata
                        .scene_assessment
                        .as_ref()
                        .is_some_and(|assessment| assessment.runnable)
            })
            .expect("expected validated animated Shine sample 3337606455");
        let item_id = item.entry.library_item_id.unwrap();
        assert!(matches!(
            DesktopService::apply_to_monitor(&monitor.id, &item_id).unwrap(),
            DesktopApplyResult::AppliedWithBackend { .. }
        ));
        let session_path = isolated.path().join("lwe/session.toml");
        let original_assignment = fs::read(&session_path).unwrap();
        let identity = desktop_apply_backend_slot()
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .handle
            .shutdown_flag();

        fn status() -> lwe_engine::EngineStatus {
            let mut guard = desktop_apply_backend_slot().lock().unwrap();
            let backend = guard.as_mut().unwrap();
            backend.handle.send(EngineCommand::GetStatus).unwrap();
            let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;
            loop {
                match DesktopService::recv_backend_event(&backend.events, deadline).unwrap() {
                    Some(EngineEvent::Status(status)) => return status,
                    Some(EngineEvent::Error(reason)) => {
                        panic!("real Settings backend error: {reason}")
                    }
                    Some(_) => {}
                    None => panic!("real Settings status timed out"),
                }
            }
        }
        let mut rates = Vec::new();
        for fps in [15, 30] {
            let saved = crate::services::settings_service::SettingsService::update_settings(
                crate::models::SettingsUpdateInput {
                    fps_limit: Some(fps),
                    pause_on_battery: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(saved.fps_limit, Some(fps));
            assert!(
                saved.playback_settings_issue.is_none(),
                "{:?}",
                saved.playback_settings_issue
            );
            let first = status();
            assert_eq!(first.playback_settings.fps_limit, Some(fps));
            let start = Instant::now();
            let mut requests = 0;
            while start.elapsed() < Duration::from_secs(10) {
                let current = status();
                assert!(current.running && current.output_errors.is_empty());
                assert_eq!(current.playback_settings.fps_limit, Some(fps));
                requests += 1;
                std::thread::sleep(Duration::from_millis(2));
            }
            let last = status();
            let seconds = start.elapsed().as_secs_f64();
            let swaps = last.render_metrics[&monitor.backend_output_id].swaps
                - first.render_metrics[&monitor.backend_output_id].swaps;
            let draws = last.render_metrics[&monitor.backend_output_id].draws
                - first.render_metrics[&monitor.backend_output_id].draws;
            let rate = swaps as f64 / seconds;
            assert!(
                swaps as f64 <= f64::from(fps) * seconds + 2.0,
                "FPS cap bypassed: {rate}"
            );
            assert!(
                rate >= f64::from(fps) * 0.7,
                "insufficient animated completed swaps: {rate}"
            );
            rates.push(rate);
            assert_eq!(fs::read(&session_path).unwrap(), original_assignment);
            assert!(Arc::ptr_eq(
                &identity,
                &desktop_apply_backend_slot()
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .handle
                    .shutdown_flag()
            ));
            let loaded = crate::services::settings_persistence_service::SettingsPersistenceService::for_user_path().unwrap().load_settings();
            assert!(
                matches!(loaded, crate::results::settings_persistence::SettingsPersistenceLoad::Loaded(settings) if settings.fps_limit == Some(fps))
            );
            eprintln!(
                "settings_live_sample {}",
                serde_json::json!({ "fps_limit": fps, "elapsed_seconds": seconds, "draws": draws, "completed_swaps": swaps, "completed_swaps_per_second": rate, "status_requests": requests, "same_backend": true, "assignment_unchanged": true, "saved_preference_verified": true })
            );
        }
        assert!(
            rates[1] > rates[0] * 1.4,
            "live FPS update did not measurably change cadence: {rates:?}"
        );
        let saved = crate::services::settings_service::SettingsService::update_settings(
            crate::models::SettingsUpdateInput {
                pause_on_battery: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(saved.pause_on_battery && saved.playback_settings_issue.is_none());
        let actual = status();
        assert!(actual.playback_settings.pause_on_battery);
        assert_eq!(actual.power_paused, actual.on_battery);
        eprintln!(
            "settings_live_power {}",
            serde_json::json!({ "pause_on_battery": true, "on_battery": actual.on_battery, "power_paused": actual.power_paused, "hardware_transition_tested": false })
        );
        assert_eq!(fs::read(&session_path).unwrap(), original_assignment);
        assert!(matches!(
            DesktopService::clear_monitor(&monitor.id).unwrap(),
            DesktopApplyResult::Cleared { .. }
        ));
        let backend = desktop_apply_backend_slot().lock().unwrap().take().unwrap();
        backend.handle.request_shutdown();
        backend
            .handle
            .join()
            .expect("Settings backend shutdown must join");
        eprintln!(
            "settings_live_complete {{\"clear_succeeded\":true,\"backend_shutdown_joined\":true,\"user_config_isolated\":true}}"
        );
    }

    #[test]
    fn desktop_apply_flow_scene_video_switch_clear_and_restore() {
        if !real_desktop_tests_enabled() {
            eprintln!(
                "skipping real Scene desktop test; set LWE_REAL_DESKTOP_TESTS=1 with isolated XDG_CONFIG_HOME"
            );
            return;
        }
        let _guard = real_desktop_flow_test_guard();
        let monitor = known_monitor().expect("expected a real monitor");
        let monitor_id = monitor.id.clone();
        let projection = LibraryService::load_projection().expect("expected local Library");
        for entry in &projection.entries {
            if entry.entry.project_type == WorkshopProjectType::Scene {
                eprintln!(
                    "Scene assessment {}: {:?}",
                    entry.entry.project_dir.display(),
                    entry.project_metadata.scene_assessment
                );
            }
        }
        let scenes: Vec<_> = projection
            .entries
            .into_iter()
            .filter(|entry| {
                entry.entry.project_type == WorkshopProjectType::Scene
                    && entry
                        .project_metadata
                        .scene_assessment
                        .as_ref()
                        .is_some_and(|assessment| assessment.runnable)
            })
            .filter_map(|entry| entry.entry.library_item_id)
            .collect();
        assert!(!scenes.is_empty(), "expected a validated native 2D Scene");
        let video = real_supported_library_item_id()
            .expect("expected a video for mixed renderer acceptance");
        let mut backend_identity = None;
        let cycles = std::env::var("LWE_REAL_DESKTOP_SCENE_CYCLES")
            .map(|value| {
                let cycles: usize = value.parse().expect("Scene cycles must be an integer");
                assert!(
                    (1..=1000).contains(&cycles),
                    "Scene cycles must be 1..=1000"
                );
                cycles
            })
            .unwrap_or(3);
        for cycle in 0..cycles {
            let item_id = &scenes[cycle % scenes.len()];
            eprintln!(
                "Scene desktop cycle {cycle}: {}",
                LibraryService::inspect_item(item_id)
                    .unwrap()
                    .entry
                    .project_dir
                    .display()
            );
            let result = DesktopService::apply_to_monitor(&monitor_id, item_id).unwrap();
            assert!(
                matches!(result, DesktopApplyResult::AppliedWithBackend { .. }),
                "Scene cycle {cycle}: {result:?}"
            );
            {
                let backend = desktop_apply_backend_slot().lock().unwrap();
                let identity = backend.as_ref().unwrap().handle.shutdown_flag();
                if let Some(expected) = &backend_identity {
                    assert!(
                        Arc::ptr_eq(expected, &identity),
                        "mixed renderer cycle restarted the backend"
                    );
                } else {
                    backend_identity = Some(identity);
                }
            }
            let page = DesktopService::load_page().unwrap();
            assert!(
                page.running_outputs.contains(&monitor.backend_output_id),
                "running={:?}, issue={:?}",
                page.running_outputs,
                page.runtime_issue
            );
            assert_eq!(page.assignments.get(&monitor_id), Some(item_id));
            if let Some(directory) = std::env::var_os("LWE_REAL_DESKTOP_CAPTURE_DIR") {
                let path = PathBuf::from(directory).join(format!("scene-desktop-{cycle}.png"));
                std::thread::sleep(Duration::from_secs(1));
                let layers = std::process::Command::new("niri")
                    .args(["msg", "--json", "layers"])
                    .output()
                    .unwrap();
                fs::write(path.with_extension("layers.json"), layers.stdout).unwrap();
                let status = std::process::Command::new("niri")
                    .args([
                        "msg",
                        "action",
                        "screenshot-screen",
                        "--show-pointer",
                        "false",
                        "--path",
                    ])
                    .arg(path)
                    .status()
                    .expect("niri screenshot command");
                assert!(status.success(), "Scene visible acceptance capture failed");
                std::thread::sleep(Duration::from_millis(300));
            }
            let result = DesktopService::apply_to_monitor(&monitor_id, &video).unwrap();
            assert!(
                matches!(result, DesktopApplyResult::AppliedWithBackend { .. }),
                "Video switch: {result:?}"
            );
            let result = DesktopService::apply_to_monitor(&monitor_id, item_id).unwrap();
            assert!(
                matches!(result, DesktopApplyResult::AppliedWithBackend { .. }),
                "Scene switch: {result:?}"
            );
            let result = DesktopService::clear_monitor(&monitor_id).unwrap();
            assert!(
                matches!(result, DesktopApplyResult::Cleared { .. }),
                "clear: {result:?}"
            );
            assert!(
                !DesktopService::load_page()
                    .unwrap()
                    .assignments
                    .contains_key(&monitor_id)
            );
        }
        let backend_identity = backend_identity.expect("Scene backend identity");
        let item_id = &scenes[0];
        assert!(matches!(
            DesktopService::apply_to_monitor(&monitor_id, item_id).unwrap(),
            DesktopApplyResult::AppliedWithBackend { .. }
        ));
        let previous_path = LibraryService::inspect_item(item_id)
            .unwrap()
            .entry
            .project_dir;

        // A presented and committed replacement remains reversible until the
        // atomic assignment write succeeds. Use an isolated failing file path.
        let candidate = DesktopService::apply_with_real_backend(&monitor, &video).unwrap();
        let fixture = tempfile::tempdir().unwrap();
        let blocker = fixture.path().join("not-a-directory");
        fs::write(&blocker, "block").unwrap();
        let persistence = DesktopPersistenceService::for_path(blocker.join("session.toml"));
        let reason = match persistence.save_assignment(&monitor_id, &video) {
            DesktopPersistenceWrite::Unavailable { reason } => reason,
            other => panic!("expected isolated persistence failure: {other:?}"),
        };
        let reason = DesktopService::rollback_persistence_failure(&monitor, candidate, reason);
        assert!(!reason.contains("Runtime rollback failed"), "{reason}");
        assert_eq!(
            real_active_wallpaper_path(&monitor.backend_output_id),
            Some(previous_path.clone())
        );
        assert_eq!(
            DesktopService::load_page()
                .unwrap()
                .assignments
                .get(&monitor_id),
            Some(item_id)
        );

        // A late commit ACK also rolls back after presentation, retaining the
        // old renderer until the receiver confirms success.
        let source = DesktopService::resolve_real_apply_source(&video).unwrap();
        let candidate = next_apply_request_id().unwrap();
        let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;
        {
            let mut backend = desktop_apply_backend_slot().lock().unwrap();
            let backend = backend.as_mut().unwrap();
            let path = source.path().to_path_buf();
            backend
                .handle
                .send(EngineCommand::ApplyWallpaper {
                    request_id: candidate,
                    deadline,
                    source,
                    output: Some(monitor.backend_output_id.clone()),
                })
                .unwrap();
            DesktopService::wait_for_apply(
                &backend.events,
                candidate,
                &monitor.backend_output_id,
                &path,
                deadline,
            )
            .unwrap();
            backend
                .handle
                .send(EngineCommand::CommitApply {
                    request_id: candidate,
                    output: monitor.backend_output_id.clone(),
                })
                .unwrap();
            backend.handle.send(EngineCommand::GetStatus).unwrap();
            // Receiving Status proves the preceding commit command was handled.
            DesktopService::wait_for_status(backend).unwrap();
        }
        // Cancellation is still reversible even though the commit was handled.
        {
            let backend = desktop_apply_backend_slot().lock().unwrap();
            DesktopService::cancel_backend_apply(
                backend.as_ref().unwrap(),
                candidate,
                &monitor.backend_output_id,
            )
            .unwrap();
        }
        assert_eq!(
            real_active_wallpaper_path(&monitor.backend_output_id),
            Some(previous_path.clone())
        );
        {
            let mut backend = desktop_apply_backend_slot().lock().unwrap();
            let old = backend.take().unwrap();
            old.handle.request_shutdown();
            old.handle.join().expect("engine shutdown");
        }
        DesktopService::restore_saved_assignments().unwrap();
        let page = DesktopService::load_page().unwrap();
        assert_eq!(page.assignments.get(&monitor_id), Some(item_id));
        assert!(
            page.running_outputs.contains(&monitor.backend_output_id),
            "restore issues: {:?}",
            page.restore_issues
        );
        let saved_generation = DesktopService::apply_with_real_backend(&monitor, item_id).unwrap();
        DesktopService::finalize_real_apply(&monitor, saved_generation);
        assert_eq!(
            real_active_wallpaper_path(&monitor.backend_output_id),
            Some(previous_path.clone())
        );
        {
            let backend = desktop_apply_backend_slot().lock().unwrap();
            DesktopService::cancel_backend_apply(
                backend.as_ref().unwrap(),
                saved_generation,
                &monitor.backend_output_id,
            )
            .unwrap();
        }
        assert_eq!(
            real_active_wallpaper_path(&monitor.backend_output_id),
            Some(previous_path.clone())
        );

        // First-frame presentation without a timely host commit expires and
        // restores the old picture even if the receiver never accepts its ACK.
        let source = DesktopService::resolve_real_apply_source(&video).unwrap();
        let candidate = next_apply_request_id().unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        {
            let backend = desktop_apply_backend_slot().lock().unwrap();
            let backend = backend.as_ref().unwrap();
            let path = source.path().to_path_buf();
            backend
                .handle
                .send(EngineCommand::ApplyWallpaper {
                    request_id: candidate,
                    deadline,
                    source,
                    output: Some(monitor.backend_output_id.clone()),
                })
                .unwrap();
            DesktopService::wait_for_apply(
                &backend.events,
                candidate,
                &monitor.backend_output_id,
                &path,
                deadline,
            )
            .unwrap();
            std::thread::sleep(
                (deadline + Duration::from_millis(50)).saturating_duration_since(Instant::now()),
            );
            DesktopService::cancel_backend_apply(backend, candidate, &monitor.backend_output_id)
                .unwrap();
        }
        assert_eq!(
            real_active_wallpaper_path(&monitor.backend_output_id),
            Some(previous_path)
        );
        {
            let backend = desktop_apply_backend_slot().lock().unwrap();
            assert!(!Arc::ptr_eq(
                &backend_identity,
                &backend.as_ref().unwrap().handle.shutdown_flag()
            ));
        }
        for output in DesktopService::load_page().unwrap().assignments.keys() {
            let result = DesktopService::clear_monitor(output).unwrap();
            assert!(
                matches!(result, DesktopApplyResult::Cleared { .. }),
                "clear all: {result:?}"
            );
        }
        assert!(DesktopService::load_page().unwrap().assignments.is_empty());
    }

    fn real_active_wallpaper_path(output: &str) -> Option<PathBuf> {
        let backend = desktop_apply_backend_slot().lock().unwrap();
        let backend = backend.as_ref().unwrap();
        backend.handle.send(EngineCommand::GetStatus).unwrap();
        let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;
        loop {
            match DesktopService::recv_backend_event(&backend.events, deadline).unwrap() {
                Some(EngineEvent::Status(status)) => {
                    return status.active_wallpapers.get(output).cloned().flatten();
                }
                Some(_) => {}
                None => panic!("runtime status timeout"),
            }
        }
    }
}
