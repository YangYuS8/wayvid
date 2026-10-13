//! Opt-in resource lifecycle diagnostics, without settings/session writes.
//! LWE_REAL_DESKTOP_TESTS=1 cargo run -p lwe-engine --example scene_resources --
//! PROJECT ASSETS VIDEO cpu|scene|video|mixed|restart CYCLES [--trim]
//!
//! Emits one JSON line per snapshot; keeps no growing in-process report. Rust
//! allocation counts exclude native Mesa/mpv allocations. RSS is not VRAM.
//! /proc and JSON sampling add fixed temporary allocations of their own.
//! GNU-only --trim is an explicit allocator-retention experiment. It samples
//! before/after malloc_trim only after clear or joined backend shutdown, never
//! during playback or CPU preparation. Normal diagnostics and production do not
//! modify allocator policy.

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail, ensure};
use lwe_engine::scene::PreparedScene;
use lwe_engine::{
    EngineCommand, EngineConfig, EngineEvent, EngineHandle, WallpaperSource, spawn_engine,
};
use serde_json::json;

struct CountedSystem;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn added(bytes: usize) {
    PEAK.fetch_max(
        LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes,
        Ordering::Relaxed,
    );
}

// Safety: forward the exact pointer/layout contract to System and count only
// successful allocations. Counters never allocate or touch returned memory.
unsafe impl GlobalAlloc for CountedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            added(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            added(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe {
            System.dealloc(pointer, layout);
        }
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(pointer, layout, size) };
        if !result.is_null() {
            if size >= layout.size() {
                added(size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - size, Ordering::Relaxed);
            }
        }
        result
    }
}

#[global_allocator]
static ALLOCATOR: CountedSystem = CountedSystem;

struct Backend {
    handle: Option<EngineHandle>,
    events: mpsc::Receiver<EngineEvent>,
    output: String,
}

impl Backend {
    fn start() -> Result<Self> {
        let (handle, events) = spawn_engine(EngineConfig {
            fps_limit: Some(30),
            ..EngineConfig::default()
        })?;
        let mut backend = Self {
            handle: Some(handle),
            events,
            output: String::new(),
        };
        backend.wait(|event| matches!(event, EngineEvent::Started))?;
        backend.send(EngineCommand::GetStatus)?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let EngineEvent::Status(status) = backend.next_until(deadline)? {
                ensure!(
                    status.outputs.len() == 1,
                    "Diagnostic requires exactly one physical output"
                );
                backend.output = status.outputs[0].name.clone();
                break;
            }
        }
        Ok(backend)
    }
    fn send(&self, command: EngineCommand) -> Result<()> {
        self.handle
            .as_ref()
            .context("Backend stopped")?
            .send(command)
    }
    fn next_until(&self, deadline: Instant) -> Result<EngineEvent> {
        ensure!(Instant::now() < deadline, "Backend operation expired");
        let event = self
            .events
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .context("Backend ACK timeout")?;
        match event {
            EngineEvent::Error(ref reason)
            | EngineEvent::OutputError { ref reason, .. }
            | EngineEvent::ApplyFailed { ref reason, .. } => bail!("Backend failure: {reason}"),
            EngineEvent::Stopped => bail!("Backend stopped unexpectedly"),
            _ => Ok(event),
        }
    }
    fn wait(&self, select: impl Fn(&EngineEvent) -> bool) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            ensure!(Instant::now() < deadline, "Backend operation expired");
            if select(&self.next_until(deadline)?) {
                return Ok(());
            }
        }
    }
    fn apply(&self, source: WallpaperSource, request_id: u64) -> Result<()> {
        self.send(EngineCommand::ApplyWallpaper {
            request_id,
            deadline: Instant::now() + Duration::from_secs(10),
            source,
            output: Some(self.output.clone()),
        })?;
        self.wait(|event| {
            matches!(event, EngineEvent::WallpaperApplied { request_id: id, output, .. }
            if *id == request_id && output == &self.output)
        })?;
        self.send(EngineCommand::CommitApply {
            request_id,
            output: self.output.clone(),
        })?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let EngineEvent::ApplyCommitted {
                request_id: id,
                output,
                result,
            } = self.next_until(deadline)?
                && id == request_id
                && output == self.output
            {
                result.map_err(anyhow::Error::msg)?;
                break;
            }
        }
        self.send(EngineCommand::FinalizeApply {
            request_id,
            output: self.output.clone(),
        })?;
        self.send(EngineCommand::GetStatus)?;
        self.wait(|event| matches!(event, EngineEvent::Status(status) if status.active_wallpapers.contains_key(&self.output)))?;
        thread::sleep(Duration::from_millis(250));
        Ok(())
    }
    fn clear(&self) -> Result<()> {
        self.send(EngineCommand::ClearWallpaper {
            output: Some(self.output.clone()),
        })?;
        self.wait(|event| matches!(event, EngineEvent::WallpaperCleared { output } if output == &self.output))?;
        self.send(EngineCommand::GetStatus)?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let EngineEvent::Status(status) = self.next_until(deadline)? {
                ensure!(
                    status.active_wallpapers.is_empty() && status.render_metrics.is_empty(),
                    "Clear retained active sessions"
                );
                break;
            }
        }
        thread::sleep(Duration::from_millis(100));
        Ok(())
    }
    fn finish(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.request_shutdown();
            handle.join()?;
        }
        Ok(())
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        if let Err(error) = self.finish() {
            eprintln!("Resource probe cleanup failed: {error:#}");
        }
    }
}

fn kib_fields(path: &str) -> Result<BTreeMap<String, u64>> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let mut fields = line.split_whitespace();
        if let (Some(key), Some(value), Some("kB")) = (fields.next(), fields.next(), fields.next())
            && let Ok(value) = value.parse::<u64>()
        {
            values.insert(key.trim_end_matches(':').to_owned(), value * 1024);
        }
    }
    Ok(values)
}

fn snapshot(cycle: usize, phase: &str) -> Result<()> {
    let rust_live = LIVE.load(Ordering::Relaxed);
    let rust_peak = PEAK.load(Ordering::Relaxed);
    // glibc's view of allocator blocks; excludes allocations made through other
    // allocators/direct mmap and must not be treated as total native live memory.
    let [used, free, mapped] = glibc_heap();
    let rss = kib_fields("/proc/self/smaps_rollup")?;
    let mut mappings = BTreeMap::<String, u64>::new();
    let mut name = String::new();
    for line in fs::read_to_string("/proc/self/smaps")?.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.first().is_some_and(|field| field.contains('-')) {
            name = match fields.get(5) {
                None => "anonymous".to_owned(),
                Some(&"[heap]") => "heap".to_owned(),
                Some(path) if path.starts_with("/dev/dri/") => "drm".to_owned(),
                Some(path) if path.starts_with('/') => "file".to_owned(),
                Some(_) => "other".to_owned(),
            };
        } else if let ["Rss:", value, "kB"] = fields.as_slice() {
            *mappings.entry(name.clone()).or_default() += value.parse::<u64>()? * 1024;
        }
    }
    let mut drm = BTreeMap::<String, BTreeMap<String, u64>>::new();
    for entry in fs::read_dir("/proc/self/fdinfo")? {
        let contents = fs::read_to_string(entry?.path()).unwrap_or_default();
        let mut client = None;
        let mut device = "unknown".to_owned();
        let mut driver = "unknown".to_owned();
        let mut memory = BTreeMap::new();
        for line in contents.lines() {
            let mut fields = line.split_whitespace();
            if let (Some(key), Some(value)) = (fields.next(), fields.next()) {
                if key == "drm-client-id:" {
                    client = Some(value.to_owned());
                } else if key == "drm-pdev:" {
                    device = value.to_owned();
                } else if key == "drm-driver:" {
                    driver = value.to_owned();
                } else if key.starts_with("drm-memory-")
                    && fields.next() == Some("KiB")
                    && let Ok(value) = value.parse::<u64>()
                {
                    memory.insert(key.trim_end_matches(':').to_owned(), value * 1024);
                }
            }
        }
        if let Some(client) = client {
            drm.insert(format!("{device}/{driver}/{client}"), memory);
        }
    }
    println!(
        "{}",
        json!({"cycle":cycle,"phase":phase,"rust_live_bytes":rust_live,"rust_peak_bytes":rust_peak,
            "glibc_used_bytes":used,"glibc_free_bytes":free,"glibc_mmap_bytes":mapped,
        "smaps_bytes":rss,"rss_mappings_bytes":mappings,"drm_clients_bytes":drm,
        "fds":fs::read_dir("/proc/self/fd")?.count(),"threads":fs::read_dir("/proc/self/task")?.count()})
    );
    Ok(())
}

#[cfg(target_env = "gnu")]
fn trim_after_release(cycle: usize, boundary: &str, enabled: bool) -> Result<()> {
    if !enabled {
        return Ok(());
    }
    snapshot(cycle, &format!("{boundary}_before_trim"))?;
    let started = Instant::now();
    // SAFETY: GNU malloc_trim releases only allocator-owned free pages, accepts
    // this pad argument and is thread-safe. No application pointer is changed.
    // The opt-in probe calls it only after its owned resources were released.
    let released_pages = unsafe { libc::malloc_trim(0) } != 0;
    let elapsed_microseconds = started.elapsed().as_micros();
    println!(
        "{}",
        json!({"cycle":cycle,"phase":"allocator_trim","boundary":boundary,
            "released_pages":released_pages,"elapsed_microseconds":elapsed_microseconds})
    );
    snapshot(cycle, &format!("{boundary}_after_trim"))
}

#[cfg(not(target_env = "gnu"))]
fn trim_after_release(_cycle: usize, _boundary: &str, enabled: bool) -> Result<()> {
    ensure!(!enabled, "--trim requires a GNU allocator build");
    Ok(())
}

#[cfg(target_env = "gnu")]
fn glibc_heap() -> [Option<usize>; 3] {
    // Read-only glibc statistics; never trim or modify allocator policy here.
    let heap = unsafe { libc::mallinfo2() };
    [Some(heap.uordblks), Some(heap.fordblks), Some(heap.hblkhd)]
}

#[cfg(not(target_env = "gnu"))]
fn glibc_heap() -> [Option<usize>; 3] {
    [None; 3]
}

fn prepare(project: &Path, assets: &Path) -> Result<WallpaperSource> {
    Ok(WallpaperSource::Scene {
        path: project.to_owned(),
        prepared: Arc::new(PreparedScene::prepare(project, Some(assets))?),
    })
}

fn run() -> Result<()> {
    ensure!(
        std::env::var("LWE_REAL_DESKTOP_TESTS").as_deref() == Ok("1"),
        "Set LWE_REAL_DESKTOP_TESTS=1 to opt in"
    );
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(
        args.len() == 5 || args.len() == 6,
        "Usage: scene_resources PROJECT ASSETS VIDEO cpu|scene|video|mixed|restart CYCLES [--trim]"
    );
    let trim = if let Some(argument) = args.get(5) {
        ensure!(argument == "--trim", "Only --trim is accepted after CYCLES");
        ensure!(
            cfg!(target_env = "gnu"),
            "--trim requires a GNU allocator build"
        );
        true
    } else {
        false
    };
    let project = PathBuf::from(&args[0]);
    let assets = PathBuf::from(&args[1]);
    let video = PathBuf::from(&args[2]);
    let mode = args[3].to_str().context("Mode must be UTF-8")?;
    ensure!(
        ["cpu", "scene", "video", "mixed", "restart"].contains(&mode),
        "Invalid mode"
    );
    ensure!(
        mode != "cpu" || !trim,
        "--trim requires a desktop lifecycle mode"
    );
    let cycles: usize = args[4].to_str().context("Cycles must be UTF-8")?.parse()?;
    ensure!((1..=200).contains(&cycles), "Cycles must be 1..=200");
    snapshot(0, "baseline")?;
    if mode == "cpu" {
        for cycle in 1..=cycles {
            let source = prepare(&project, &assets)?;
            snapshot(cycle, "prepared")?;
            drop(source);
            snapshot(cycle, "cpu_released")?;
        }
    } else {
        let mut backend = Backend::start()?;
        snapshot(0, "backend_started")?;
        for cycle in 1..=cycles {
            if mode != "video" {
                backend.apply(prepare(&project, &assets)?, cycle as u64 * 3)?;
                snapshot(cycle, "scene_applied")?;
            }
            if mode == "video" || mode == "mixed" {
                backend.apply(
                    WallpaperSource::Video {
                        path: video.clone(),
                    },
                    cycle as u64 * 3 + 1,
                )?;
                snapshot(cycle, "video_applied")?;
            }
            if mode == "mixed" {
                backend.apply(prepare(&project, &assets)?, cycle as u64 * 3 + 2)?;
                snapshot(cycle, "scene_reapplied")?;
            }
            backend.clear()?;
            snapshot(cycle, "cleared")?;
            trim_after_release(cycle, "cleared", trim)?;
            if mode == "restart" {
                backend.finish()?;
                drop(backend);
                snapshot(cycle, "backend_stopped")?;
                trim_after_release(cycle, "backend_stopped", trim)?;
                backend = Backend::start()?;
            }
        }
        backend.finish()?;
        drop(backend);
    }
    thread::sleep(Duration::from_secs(1));
    snapshot(cycles, "finished")?;
    trim_after_release(cycles, "finished", trim)?;
    eprintln!(
        "Resource probe completed: {mode}, {cycles} cycles; backend joined; no persistent writes"
    );
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Resource probe failed: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
