//! Opt-in engine-only desktop measurement; never writes settings/session files.
//! LWE_REAL_DESKTOP_TESTS=1 cargo run -p lwe-engine --example scene_probe --
//! PROJECT ASSETS [OUTPUT] [DURATION_SECONDS] [--static STATIC_PROJECT] [--properties JSON_FILE]

use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail, ensure};
use lwe_engine::engine::EngineRenderMetrics;
use lwe_engine::scene::PreparedScene;
use lwe_engine::{
    EngineCommand, EngineConfig, EngineEvent, EngineHandle, EngineStatus, WallpaperSource,
    spawn_engine,
};
use serde_json::{Value, json};

const FPS_LIMIT: u32 = 30;
const OPERATION_TIMEOUT: Duration = Duration::from_secs(10);
const STATUS_INTERVAL: Duration = Duration::from_millis(2);

struct Options {
    project: PathBuf,
    assets: PathBuf,
    output: Option<String>,
    duration: Duration,
    static_project: Option<PathBuf>,
    properties: std::collections::BTreeMap<String, Value>,
}

impl Options {
    fn parse() -> Result<Self> {
        let mut args: VecDeque<_> = std::env::args_os().skip(1).collect();
        let usage = "Usage: scene_probe PROJECT ASSETS [OUTPUT] [DURATION_SECONDS] [--static STATIC_PROJECT] [--properties JSON_FILE]";
        let project = args.pop_front().map(PathBuf::from).context(usage)?;
        let assets = args.pop_front().map(PathBuf::from).context(usage)?;
        let mut static_project = None;
        let mut properties = std::collections::BTreeMap::new();
        let mut positional = Vec::new();
        while let Some(arg) = args.pop_front() {
            if arg == "--properties" {
                let file = args
                    .pop_front()
                    .map(PathBuf::from)
                    .context("--properties requires a JSON path")?;
                use std::io::Read;
                let mut bytes = Vec::new();
                std::fs::File::open(file)?
                    .take(64 * 1024 + 1)
                    .read_to_end(&mut bytes)?;
                ensure!(
                    bytes.len() <= 64 * 1024,
                    "Probe property file exceeds limit"
                );
                properties = serde_json::from_slice(&bytes)?;
            } else if arg == "--static" {
                ensure!(static_project.is_none(), "Duplicate --static option");
                static_project = Some(
                    args.pop_front()
                        .map(PathBuf::from)
                        .context("--static requires a project directory")?,
                );
            } else {
                let arg = arg
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("Output/duration argument must be UTF-8"))?;
                ensure!(!arg.starts_with("--"), "Unknown option {arg}");
                positional.push(arg);
            }
        }
        ensure!(positional.len() <= 2, "{usage}");
        let (output, seconds) = match positional.as_slice() {
            [] => (None, 60.0),
            [one] => match one.parse::<f64>() {
                Ok(seconds) => (None, seconds),
                Err(_) => (Some(one.clone()), 60.0),
            },
            [output, duration] => (
                Some(output.clone()),
                duration.parse::<f64>().context("Invalid duration")?,
            ),
            _ => unreachable!(),
        };
        ensure!(
            seconds.is_finite() && seconds > 0.0 && seconds <= 600.0,
            "Measurement duration must be between 0 and 600 seconds"
        );
        Ok(Self {
            project,
            assets,
            output,
            duration: Duration::from_secs_f64(seconds),
            static_project,
            properties,
        })
    }
}

/// Owns only this process's new backend. Result errors and panic unwinding both
/// shut it down and join before returning; the user's persistent state is unused.
struct ProbeEngine {
    handle: Option<EngineHandle>,
    events: mpsc::Receiver<EngineEvent>,
}

impl ProbeEngine {
    fn start() -> Result<Self> {
        let (handle, events) = spawn_engine(EngineConfig {
            fps_limit: Some(FPS_LIMIT),
            ..EngineConfig::default()
        })?;
        let engine = Self {
            handle: Some(handle),
            events,
        };
        engine.wait(|event| match event {
            EngineEvent::Started => Some(Ok(())),
            _ => None,
        })?;
        Ok(engine)
    }

    fn send(&self, command: EngineCommand) -> Result<()> {
        self.handle
            .as_ref()
            .context("Probe backend has stopped")?
            .send(command)
    }

    fn wait<T>(&self, mut select: impl FnMut(EngineEvent) -> Option<Result<T>>) -> Result<T> {
        let deadline = Instant::now() + OPERATION_TIMEOUT;
        loop {
            let event = self
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .context("Timed out or disconnected while waiting for the probe backend")?;
            match event {
                EngineEvent::Error(reason)
                | EngineEvent::OutputError { reason, .. }
                | EngineEvent::ApplyFailed { reason, .. } => {
                    bail!("Probe backend failed: {reason}")
                }
                EngineEvent::Stopped => bail!("Probe backend stopped unexpectedly"),
                event => {
                    if let Some(result) = select(event) {
                        return result;
                    }
                }
            }
        }
    }

    fn status(&self) -> Result<EngineStatus> {
        self.send(EngineCommand::GetStatus)?;
        let status = self.wait(|event| match event {
            EngineEvent::Status(status) => Some(Ok(status)),
            _ => None,
        })?;
        ensure!(status.running, "Probe backend is not running");
        ensure!(
            status.output_errors.is_empty(),
            "Probe output errors: {:?}",
            status.output_errors
        );
        Ok(status)
    }

    fn metrics(&self, output: &str) -> Result<EngineRenderMetrics> {
        self.status()?
            .render_metrics
            .get(output)
            .copied()
            .context("Probe output has no render metrics")
    }

    fn apply(
        &self,
        output: &str,
        project: &std::path::Path,
        prepared: Arc<PreparedScene>,
        request_id: u64,
    ) -> Result<()> {
        self.send(EngineCommand::ApplyWallpaper {
            request_id,
            deadline: Instant::now() + OPERATION_TIMEOUT,
            source: WallpaperSource::Scene {
                path: project.to_path_buf(),
                prepared,
            },
            output: Some(output.to_owned()),
        })?;
        self.wait(|event| match event {
            EngineEvent::WallpaperApplied {
                request_id: id,
                output: name,
                ..
            } if id == request_id && name == output => Some(Ok(())),
            _ => None,
        })?;
        self.send(EngineCommand::CommitApply {
            request_id,
            output: output.to_owned(),
        })?;
        self.wait(|event| match event {
            EngineEvent::ApplyCommitted {
                request_id: id,
                output: name,
                result,
            } if id == request_id && name == output => Some(result.map_err(anyhow::Error::msg)),
            _ => None,
        })?;
        // Simulate the host's acceptance; this standalone engine probe does not
        // save an assignment and does not establish persistence/UI acceptance.
        self.send(EngineCommand::FinalizeApply {
            request_id,
            output: output.to_owned(),
        })?;
        ensure!(
            self.status()?
                .active_wallpapers
                .get(output)
                .and_then(Option::as_deref)
                == Some(project),
            "Probe assignment did not become current"
        );
        Ok(())
    }

    fn clear(&self, output: &str) -> Result<()> {
        self.send(EngineCommand::ClearWallpaper {
            output: Some(output.to_owned()),
        })?;
        self.wait(|event| match event {
            EngineEvent::WallpaperCleared { output: name } if name == output => Some(Ok(())),
            _ => None,
        })?;
        let status = self.status()?;
        ensure!(
            !status.render_metrics.contains_key(output)
                && !status.active_wallpapers.contains_key(output),
            "Clear retained output metrics or assignment"
        );
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

impl Drop for ProbeEngine {
    fn drop(&mut self) {
        if let Err(error) = self.finish() {
            eprintln!("Probe backend cleanup failed: {error:#}");
        }
    }
}

fn counters_equal(left: EngineRenderMetrics, right: EngineRenderMetrics) -> bool {
    left == right
}

fn settle_idle(engine: &ProbeEngine, output: &str) -> Result<EngineRenderMetrics> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut previous = engine.metrics(output)?;
    let mut unchanged_since = Instant::now();
    loop {
        thread::sleep(Duration::from_millis(100));
        let current = engine.metrics(output)?;
        if counters_equal(previous, current) {
            if unchanged_since.elapsed() >= Duration::from_secs(1) {
                return Ok(current);
            }
        } else {
            unchanged_since = Instant::now();
            previous = current;
        }
        ensure!(
            Instant::now() < deadline,
            "Output did not settle into idle within five seconds"
        );
    }
}

fn delta(after: u64, before: u64) -> Result<u64> {
    after
        .checked_sub(before)
        .context("Output metrics reset during a measurement")
}

fn measure(
    engine: &ProbeEngine,
    output: &str,
    name: &str,
    duration: Duration,
    idle: bool,
) -> Result<Value> {
    eprintln!(
        "{name}: measuring {:.1} seconds with frequent GetStatus traffic",
        duration.as_secs_f64()
    );
    let before = engine.metrics(output)?;
    let start = Instant::now();
    let mut after = before;
    let mut requests = 0_u64;
    let mut last_progress = Instant::now();
    while start.elapsed() < duration {
        after = engine.metrics(output)?;
        requests += 1;
        if last_progress.elapsed() >= Duration::from_secs(10) {
            eprintln!(
                "{name}: {:.1}s, {} completed swaps, {requests} status requests",
                start.elapsed().as_secs_f64(),
                delta(after.swaps, before.swaps)?
            );
            last_progress = Instant::now();
        }
        thread::sleep(STATUS_INTERVAL.min(duration.saturating_sub(start.elapsed())));
    }
    let elapsed = start.elapsed().as_secs_f64();
    let draws = delta(after.draws, before.draws)?;
    let swaps = delta(after.swaps, before.swaps)?;
    let attempts = delta(after.render_attempts, before.render_attempts)?;
    let empty = delta(after.empty_frames, before.empty_frames)?;
    let requested = delta(after.callbacks_requested, before.callbacks_requested)?;
    let received = delta(after.callbacks_received, before.callbacks_received)?;
    let render_swap_micros = delta(
        after.frame_duration_total_micros,
        before.frame_duration_total_micros,
    )?;
    let render_swap_buckets = after
        .frame_duration_buckets
        .iter()
        .zip(before.frame_duration_buckets)
        .map(|(after, before)| delta(*after, before))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        before.scene_payload_bytes == after.scene_payload_bytes,
        "Logical Scene payload changed during stable measurement"
    );
    if idle {
        ensure!(
            draws == 0
                && swaps == 0
                && attempts == 0
                && empty == 0
                && requested == 0
                && received == 0,
            "{name} sustained a render/callback loop: before={before:?}, after={after:?}"
        );
    } else {
        ensure!(swaps > 0, "{name} produced no stable animation frames");
        ensure!(
            swaps <= (elapsed * f64::from(FPS_LIMIT)).ceil() as u64 + 1,
            "{name} exceeded the {FPS_LIMIT} FPS cap under GetStatus traffic: {} swaps in {elapsed:.3}s",
            swaps
        );
    }
    let state = engine.status()?;
    let budget = state.scene_gpu_budget;
    Ok(json!({
        "phase": name, "duration_seconds": elapsed, "status_requests": requests,
        "status_requests_per_second": requests as f64 / elapsed,
        "draws": draws, "swaps": swaps, "swaps_per_second": swaps as f64 / elapsed,
        "render_attempts": attempts, "empty_frames": empty,
        "render_swap_total_micros": render_swap_micros,
        "render_swap_mean_millis": (swaps > 0).then(|| render_swap_micros as f64 / swaps as f64 / 1000.0),
        "render_swap_buckets": render_swap_buckets,
        "lifetime_render_swap_max_micros": after.frame_duration_max_micros,
        "callbacks_requested": requested, "callbacks_received": received,
        "scene_payload_bytes": after.scene_payload_bytes, "idle_expected": idle,
        "scene_gpu_budget":{"context_generation":budget.context_generation,
            "immutable_resident_bytes":budget.immutable_resident_bytes,
            "private_and_reserved_bytes":budget.private_and_reserved_bytes,
            "immutable_reserved_bytes":budget.immutable_reserved_bytes,
            "retired_bytes":budget.retired_bytes,"logical_peak_bytes":budget.logical_peak_bytes,
            "allocation_count":budget.allocation_count,"lease_count":budget.lease_count},
        "scene_decoded_texture_bytes":state.scene_decoded_texture_bytes,
    }))
}

fn record(report: &mut Value, phase: Value) {
    report["phases"].as_array_mut().unwrap().push(phase);
}

fn run(report: &mut Value) -> Result<()> {
    ensure!(
        std::env::var("LWE_REAL_DESKTOP_TESTS").as_deref() == Ok("1"),
        "Set LWE_REAL_DESKTOP_TESTS=1 to authorize this real desktop engine probe"
    );
    let options = Options::parse()?;
    eprintln!(
        "Engine-only desktop probe: applies/clears its own backend; HOME is retained; no session/settings writes; UI and persistence are not tested."
    );
    let prepared = Arc::new(PreparedScene::prepare_with_properties(
        &options.project,
        Some(&options.assets),
        &options.properties,
    )?);
    ensure!(
        prepared.summary().animated,
        "The primary project must be animated; use --static for a separate static project"
    );
    let static_prepared = options
        .static_project
        .as_ref()
        .map(|project| {
            let prepared = PreparedScene::prepare(project, Some(&options.assets))?;
            ensure!(
                !prepared.summary().animated,
                "--static project must have no animated effects"
            );
            Ok::<_, anyhow::Error>(Arc::new(prepared))
        })
        .transpose()?;
    let mut engine = ProbeEngine::start()?;
    let outputs = engine.status()?.outputs;
    let output = if let Some(output) = options.output {
        ensure!(
            outputs.iter().any(|item| item.name == output),
            "Output {output} is unavailable"
        );
        output
    } else {
        ensure!(
            outputs.len() == 1,
            "Specify OUTPUT when the backend does not expose exactly one monitor"
        );
        outputs[0].name.clone()
    };
    report["output"] = json!(output);
    report["project"] = json!(options.project.display().to_string());
    report["fps_limit"] = json!(FPS_LIMIT);
    report["requested_phase_seconds"] = json!(options.duration.as_secs_f64());
    engine.apply(&output, &options.project, prepared, 1)?;
    thread::sleep(Duration::from_secs(1)); // Exclude initial presentation from pacing.
    record(
        report,
        measure(&engine, &output, "animated", options.duration, false)?,
    );
    engine.send(EngineCommand::Pause {
        output: Some(output.clone()),
    })?;
    settle_idle(&engine, &output)?;
    record(
        report,
        measure(&engine, &output, "paused", options.duration, true)?,
    );
    engine.send(EngineCommand::Resume {
        output: Some(output.clone()),
    })?;
    thread::sleep(Duration::from_secs(1));
    record(
        report,
        measure(&engine, &output, "resumed", Duration::from_secs(3), false)?,
    );
    if let (Some(project), Some(prepared)) = (options.static_project, static_prepared) {
        engine.send(EngineCommand::Pause {
            output: Some(output.clone()),
        })?;
        let before = settle_idle(&engine, &output)?;
        engine.apply(&output, &project, prepared, 2)?;
        let after = engine.metrics(&output)?;
        ensure!(
            delta(after.draws, before.draws)? == 1 && delta(after.swaps, before.swaps)? == 1,
            "Static candidate did not present exactly one frame while paused"
        );
        record(
            report,
            json!({"phase":"static_first_frame_while_paused", "draws":1, "swaps":1}),
        );
        engine.send(EngineCommand::Resume {
            output: Some(output.clone()),
        })?;
        settle_idle(&engine, &output)?;
        record(
            report,
            measure(&engine, &output, "static_playing", options.duration, true)?,
        );
    }
    engine.clear(&output)?;
    record(
        report,
        json!({"phase":"cleared", "output_metrics_removed":true}),
    );
    engine.finish()?;
    report["backend_shutdown_joined"] = json!(true);
    Ok(())
}

fn main() -> ExitCode {
    let mut report =
        json!({"probe":"engine_only", "writes_session":false, "tests_ui":false, "phases":[]});
    let result = run(&mut report);
    report["ok"] = json!(result.is_ok());
    if let Err(error) = &result {
        report["error"] = json!(format!("{error:#}"));
    }
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    if result.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
