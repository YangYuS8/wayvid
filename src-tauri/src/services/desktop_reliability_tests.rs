//! Opt-in, real production-service acceptance. The supervisor owns an isolated
//! config root and records each NDJSON event durably outside the repository.
use super::*;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn emit(value: serde_json::Value) {
    println!("{value}");
    std::io::stdout().flush().expect("flush acceptance event");
}

fn number(name: &str, default: u64, maximum: u64) -> u64 {
    let value = std::env::var(name)
        .map(|value| value.parse::<u64>().expect("invalid acceptance option"))
        .unwrap_or(default);
    assert!(value <= maximum, "acceptance option exceeds its bound");
    value
}

fn status() -> lwe_engine::EngineStatus {
    let mut guard = desktop_apply_backend_slot().lock().unwrap();
    let backend = guard.as_mut().expect("acceptance backend missing");
    backend.handle.send(EngineCommand::GetStatus).unwrap();
    let deadline = Instant::now() + REAL_APPLY_BACKEND_TIMEOUT;
    loop {
        match DesktopService::recv_backend_event(&backend.events, deadline).unwrap() {
            Some(EngineEvent::Status(state)) => return state,
            Some(
                EngineEvent::Error(reason)
                | EngineEvent::OutputError { reason, .. }
                | EngineEvent::ApplyFailed { reason, .. },
            ) => panic!("acceptance backend: {reason}"),
            Some(EngineEvent::Stopped) | None => panic!("acceptance status disconnected/timed out"),
            Some(_) => {}
        }
    }
}

fn command(command: EngineCommand) {
    desktop_apply_backend_slot()
        .lock()
        .unwrap()
        .as_ref()
        .expect("acceptance backend missing")
        .handle
        .send(command)
        .unwrap();
}

fn sample(output: &str, item_id: &str, phase: &str) -> lwe_engine::engine::EngineRenderMetrics {
    let state = status();
    assert!(state.running && state.output_errors.is_empty(), "{state:?}");
    let metrics = *state
        .render_metrics
        .get(output)
        .expect("active render metrics");
    #[cfg(target_env = "gnu")]
    let resources = {
        // Read-only allocator counters; this test never changes malloc policy.
        let heap = unsafe { libc::mallinfo2() };
        json!({"glibc_used_bytes":heap.uordblks,"glibc_free_bytes":heap.fordblks,
            "glibc_mmap_bytes":heap.hblkhd})
    };
    #[cfg(not(target_env = "gnu"))]
    let resources = serde_json::Value::Null;
    emit(json!({"event":"sample","output":output,"item_id":item_id,
        "phase":phase,"finalized":true,"healthy":true,"backend_generation":1,
        "renderer_generation":NEXT_APPLY_REQUEST.load(Ordering::Relaxed).saturating_sub(1),
        "metrics":{"swaps":metrics.swaps,"draws":metrics.draws,
            "render_attempts":metrics.render_attempts,"callbacks_requested":metrics.callbacks_requested,
            "callbacks_received":metrics.callbacks_received,"scene_payload_bytes":metrics.scene_payload_bytes,
            "frame_duration_buckets":metrics.frame_duration_buckets,
            "frame_duration_total_micros":metrics.frame_duration_total_micros,
            "frame_duration_max_micros":metrics.frame_duration_max_micros},
        "resources":resources}));
    metrics
}

struct Cleanup(String);
impl Drop for Cleanup {
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

#[test]
#[ignore = "real desktop, long duration, isolated config and explicit supervisor required"]
fn desktop_scene_daily_use_reliability() {
    assert_eq!(std::env::var("LWE_REAL_DESKTOP_TESTS").as_deref(), Ok("1"));
    let root = PathBuf::from(std::env::var_os("XDG_CONFIG_HOME").expect("isolated config root"));
    assert!(root.is_absolute());
    let home = PathBuf::from(std::env::var_os("HOME").expect("HOME for Steam discovery"));
    assert_ne!(
        root,
        home.join(".config"),
        "refusing default user configuration"
    );
    assert!(
        !root.join("lwe/session.toml").exists(),
        "require fresh acceptance state"
    );
    let _guard = real_desktop_flow_test_guard();
    assert!(desktop_apply_backend_slot().lock().unwrap().is_none());
    let monitor = match MonitorService::list_monitors_uncached() {
        MonitorDiscoveryResult::Known(mut monitors) if monitors.len() == 1 => monitors.remove(0),
        other => panic!("requires one real physical output: {other:?}"),
    };
    let _cleanup = Cleanup(monitor.id.clone());
    let ids = std::env::var("LWE_RELIABILITY_SCENE_IDS")
        .unwrap_or_else(|_| "2903458386,3659880761,3337606455,3258032485".to_string());
    let projection = LibraryService::load_projection().expect("load actual Workshop Library");
    let scenes: Vec<_> = ids
        .split(',')
        .map(|id| {
            let entry = projection
                .entries
                .iter()
                .find(|entry| {
                    entry
                        .entry
                        .project_dir
                        .file_name()
                        .is_some_and(|name| name == id)
                })
                .unwrap_or_else(|| panic!("required sample {id} missing"));
            assert!(
                entry
                    .project_metadata
                    .scene_assessment
                    .as_ref()
                    .is_some_and(|a| a.runnable),
                "required sample {id} blocked"
            );
            (
                entry.entry.library_item_id.clone().expect("Scene item ID"),
                entry.entry.project_dir.clone(),
            )
        })
        .collect();
    let required_video = std::env::var_os("LWE_RELIABILITY_VIDEO_FILE").map(|path| {
        PathBuf::from(path)
            .canonicalize()
            .expect("required video file")
    });
    let video_entry = projection
        .entries
        .iter()
        .find(|entry| {
            entry.entry.project_type == WorkshopProjectType::Video
                && WeProject::load(&entry.entry.project_dir)
                    .ok()
                    .and_then(|project| project.main_file(&entry.entry.project_dir))
                    .is_some_and(|path| {
                        path.is_file()
                            && required_video.as_ref().is_none_or(|expected| {
                                path.canonicalize().ok().as_ref() == Some(expected)
                            })
                    })
        })
        .expect("actual video sample");
    let video = video_entry
        .entry
        .library_item_id
        .clone()
        .expect("video item ID");
    let cycles = number("LWE_RELIABILITY_SWITCH_CYCLES", 100, 1000);
    let seconds = number("LWE_RELIABILITY_SOAK_SECONDS", 86400, 604800);
    let interval = number("LWE_RELIABILITY_SAMPLE_SECONDS", 30, 60).max(1);
    let stop_file = std::env::var_os("LWE_RELIABILITY_STOP_FILE").map(PathBuf::from);
    if let Some(path) = &stop_file {
        assert_eq!(
            path.parent(),
            Some(root.as_path()),
            "stop marker must be in isolated config root"
        );
        assert!(!path.exists(), "require a fresh stop marker");
    }
    let run_limit = if stop_file.is_some() {
        number("LWE_RELIABILITY_MAX_SECONDS", 604800, 604800)
    } else {
        seconds
    };
    crate::services::settings_service::SettingsService::update_settings(
        crate::models::SettingsUpdateInput {
            fps_limit: Some(30),
            pause_on_battery: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    let apply = |item: &str, path: Option<&Path>| {
        assert!(matches!(
            DesktopService::apply_to_monitor(&monitor.id, item).unwrap(),
            DesktopApplyResult::AppliedWithBackend { .. }
        ));
        let page = DesktopService::load_page().unwrap();
        assert_eq!(
            page.assignments.get(&monitor.id).map(String::as_str),
            Some(item)
        );
        assert!(page.running_outputs.contains(&monitor.backend_output_id));
        assert!(page.runtime_issue.is_none() && page.persistence_issue.is_none());
        assert!(
            matches!(DesktopPersistenceService::for_user_path().unwrap().load_state(),
            DesktopPersistenceLoad::Loaded(assignments) if assignments.get(&monitor.id).map(String::as_str)==Some(item))
        );
        if let Some(path) = path {
            assert_eq!(
                status()
                    .active_wallpapers
                    .get(&monitor.backend_output_id)
                    .and_then(Option::as_deref),
                Some(path)
            );
        }
    };
    apply(&scenes[0].0, Some(&scenes[0].1));
    let identity = desktop_apply_backend_slot()
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .handle
        .shutdown_flag();
    emit(
        json!({"event":"started","pid":std::process::id(),"output":monitor.backend_output_id,
        "backend_generation":1,"fps_limit":30,"host":"DesktopService",
        "scene_projects":scenes.iter().map(|(_,path)|path).collect::<Vec<_>>(),
        "video_project":video_entry.entry.project_dir,"switch_cycles":cycles,"soak_seconds":seconds}),
    );
    for cycle in 1..=cycles {
        let (scene, path) = &scenes[(cycle as usize - 1) % scenes.len()];
        apply(scene, Some(path));
        let first = sample(&monitor.backend_output_id, scene, "scene");
        std::thread::sleep(Duration::from_secs(1));
        assert!(sample(&monitor.backend_output_id, scene, "scene").swaps > first.swaps);
        apply(&video, None);
        emit(json!({"event":"switch","cycle":cycle,"from":"scene","to":"video","finalized":true}));
        let first = sample(&monitor.backend_output_id, &video, "video");
        std::thread::sleep(Duration::from_secs(1));
        assert!(sample(&monitor.backend_output_id, &video, "video").swaps > first.swaps);
        apply(scene, Some(path));
        emit(json!({"event":"switch","cycle":cycle,"from":"video","to":"scene","finalized":true}));
        assert!(
            Arc::ptr_eq(
                &identity,
                &desktop_apply_backend_slot()
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .handle
                    .shutdown_flag()
            ),
            "backend changed during switches"
        );
    }
    apply(&scenes[0].0, Some(&scenes[0].1));
    let saved = fs::read(root.join("lwe/session.toml")).unwrap();
    command(EngineCommand::Pause {
        output: Some(monitor.backend_output_id.clone()),
    });
    std::thread::sleep(Duration::from_secs(1));
    let paused = sample(&monitor.backend_output_id, &scenes[0].0, "paused");
    std::thread::sleep(Duration::from_secs(5));
    let after = sample(&monitor.backend_output_id, &scenes[0].0, "paused");
    assert_eq!(after.swaps, paused.swaps);
    assert_eq!(after.draws, paused.draws);
    assert_eq!(after.callbacks_requested, paused.callbacks_requested);
    command(EngineCommand::Resume {
        output: Some(monitor.backend_output_id.clone()),
    });
    emit(json!({"event":"soak_started","item_id":scenes[0].0,
        "required_active_seconds":seconds,"maximum_runtime_seconds":run_limit}));
    let start = Instant::now();
    let mut previous = sample(&monitor.backend_output_id, &scenes[0].0, "scene");
    while start.elapsed() < Duration::from_secs(run_limit) {
        if stop_file
            .as_ref()
            .is_some_and(|path| fs::read(path).ok().as_deref() == Some(b"stop\n"))
        {
            break;
        }
        std::thread::sleep(
            Duration::from_secs(interval)
                .min(Duration::from_secs(run_limit).saturating_sub(start.elapsed())),
        );
        let current = sample(&monitor.backend_output_id, &scenes[0].0, "scene");
        // A lock may prevent presentation during true suspend testing. The
        // supervisor excludes such intervals instead of crediting them as soak.
        assert!(current.swaps >= previous.swaps);
        assert_eq!(current.scene_payload_bytes, previous.scene_payload_bytes);
        assert_eq!(fs::read(root.join("lwe/session.toml")).unwrap(), saved);
        previous = current;
    }
    assert!(matches!(
        DesktopService::clear_monitor(&monitor.id).unwrap(),
        DesktopApplyResult::Cleared { .. }
    ));
    let backend = desktop_apply_backend_slot().lock().unwrap().take().unwrap();
    backend.handle.request_shutdown();
    backend.handle.join().expect("acceptance backend joined");
    emit(
        json!({"event":"stopped","healthy":true,"shutdown_joined":true,
        "cleared_isolated_assignment":true,"completed_switch_cycles":cycles,
        "requested_soak_seconds":seconds,"elapsed_monotonic_seconds":start.elapsed().as_secs_f64()}),
    );
}
