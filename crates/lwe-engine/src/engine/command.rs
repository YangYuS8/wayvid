//! Engine commands, events, and configuration types

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use lwe_core::OutputInfo;

use crate::mpv::VideoConfig;
use crate::scene::PreparedScene;

/// Validated CPU-only input. Scene I/O and decoding happen before dispatch.
#[derive(Debug, Clone)]
pub enum WallpaperSource {
    Video {
        path: PathBuf,
    },
    Scene {
        path: PathBuf,
        prepared: Arc<PreparedScene>,
    },
}

impl WallpaperSource {
    pub fn path(&self) -> &Path {
        match self {
            Self::Video { path } | Self::Scene { path, .. } => path,
        }
    }

    pub fn is_scene(&self) -> bool {
        matches!(self, Self::Scene { .. })
    }
}

/// Commands sent from GUI to engine
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// Apply wallpaper to output(s)
    ApplyWallpaper {
        /// Unique application request generation
        request_id: u64,
        /// Latest time at which the first rendered frame may be accepted
        deadline: Instant,
        /// Validated video or prepared Scene resources
        source: WallpaperSource,
        /// Target output (None = all outputs)
        output: Option<String>,
    },

    /// Accept a first-frame acknowledgement observed before its deadline.
    CommitApply { request_id: u64, output: String },

    /// Saved state now agrees with the candidate; release its rollback resources.
    FinalizeApply { request_id: u64, output: String },

    /// Cancel only the application generation still owning this output
    CancelApply { request_id: u64, output: String },

    /// Clear wallpaper from output(s)
    ClearWallpaper {
        /// Target output (None = all outputs)
        output: Option<String>,
    },

    /// Set volume for an output
    SetVolume {
        /// Target output
        output: String,
        /// Volume level (0.0 - 1.0)
        volume: f32,
    },

    /// Pause playback
    Pause {
        /// Target output (None = all outputs)
        output: Option<String>,
    },

    /// Resume playback
    Resume {
        /// Target output (None = all outputs)
        output: Option<String>,
    },

    /// Update playback policy without replacing outputs or their assignments.
    UpdatePlaybackSettings {
        settings: PlaybackSettings,
        deadline: Instant,
        reply: std::sync::mpsc::Sender<Result<(), String>>,
    },

    /// Request current outputs list
    GetOutputs,

    /// Request current status
    GetStatus,

    /// Shutdown the engine
    Shutdown,
}

/// Events sent from engine to GUI
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// Engine has started successfully
    Started,

    /// Engine has stopped
    Stopped,

    /// New output detected
    OutputAdded(OutputInfo),

    /// Output removed
    OutputRemoved(String),

    /// Current outputs list (response to GetOutputs)
    OutputsList(Vec<OutputInfo>),

    /// Current status (response to GetStatus)
    Status(EngineStatus),

    /// Wallpaper applied successfully
    WallpaperApplied {
        /// Application request that produced this first frame
        request_id: u64,
        /// Output name
        output: String,
        /// Wallpaper path
        path: PathBuf,
    },

    /// The candidate is current and remains reversible until finalization.
    ApplyCommitted {
        request_id: u64,
        output: String,
        result: Result<(), String>,
    },

    /// An application request failed on a specific output
    ApplyFailed {
        request_id: u64,
        output: String,
        reason: String,
    },

    /// Cancellation was processed, including any resource-release failure
    ApplyCancelled {
        request_id: u64,
        output: String,
        result: Result<(), String>,
    },

    /// Wallpaper cleared
    WallpaperCleared {
        /// Output name
        output: String,
    },

    /// A non-application operation failed on a specific output
    OutputError { output: String, reason: String },

    /// Error occurred
    Error(String),
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Video playback configuration
    pub video: VideoConfig,
    /// Auto-start playback when wallpaper is applied
    pub auto_play: bool,
    /// FPS limit (None = vsync/unlimited)
    pub fps_limit: Option<u32>,
    /// Pause playback when on battery power
    pub pause_on_battery: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlaybackSettings {
    pub fps_limit: Option<u32>,
    pub pause_on_battery: bool,
}

impl PlaybackSettings {
    pub fn validate(self) -> Result<(), String> {
        if self.fps_limit.is_some_and(|fps| !(1..=240).contains(&fps)) {
            return Err("Wallpaper FPS limit must be between 1 and 240".to_string());
        }
        Ok(())
    }
}

impl EngineConfig {
    pub fn playback_settings(&self) -> PlaybackSettings {
        PlaybackSettings {
            fps_limit: self.fps_limit,
            pause_on_battery: self.pause_on_battery,
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            video: VideoConfig::default(),
            auto_play: true,
            fps_limit: None,
            pause_on_battery: false,
        }
    }
}

/// Current engine status
#[derive(Debug, Clone, Default)]
pub struct EngineStatus {
    /// Whether engine is running
    pub running: bool,
    pub scene_gpu_budget: SceneGpuBudgetSnapshot,
    /// Actual retained decoded RGBA Vec capacities, deduplicated by PreparedScene
    /// Arc identity. Equal content in separate preparations remains separate.
    pub scene_decoded_texture_bytes: u64,
    pub playback_settings: PlaybackSettings,
    pub on_battery: bool,
    pub power_paused: bool,
    /// Available outputs
    pub outputs: Vec<OutputInfo>,
    /// Active wallpapers per output
    pub active_wallpapers: HashMap<String, Option<PathBuf>>,
    /// Persistent per-output runtime failures, including events not awaited by a caller.
    pub output_errors: HashMap<String, String>,
    /// Cumulative counters for each current output session. Compare snapshots
    /// over a fixed interval to measure pacing and static/paused inactivity.
    pub render_metrics: HashMap<String, EngineRenderMetrics>,
}

/// Engine-global logical allocations, not measured driver VRAM. Output metrics
/// can reference the same immutable allocation and must not be summed instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SceneGpuBudgetSnapshot {
    pub context_generation: u64,
    pub immutable_resident_bytes: u64,
    pub private_and_reserved_bytes: u64,
    pub immutable_reserved_bytes: u64,
    pub retired_bytes: u64,
    pub logical_peak_bytes: u64,
    pub allocation_count: u64,
    pub lease_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EngineRenderMetrics {
    /// Calls to an active renderer; idle policy decisions are excluded.
    pub render_attempts: u64,
    /// Completed frames drawn, rather than individual OpenGL draw calls.
    pub draws: u64,
    /// Successfully completed EGL swaps.
    pub swaps: u64,
    pub empty_frames: u64,
    pub callbacks_requested: u64,
    pub callbacks_received: u64,
    /// Referenced logical RGBA Scene textures (unique within this output) plus
    /// private FBOs/geometry for candidate and rollback. Shared allocations may
    /// appear on other outputs; use scene_gpu_budget for the engine total.
    /// Excludes driver overhead, EGL buffers and video; this is not VRAM usage.
    pub scene_payload_bytes: u64,
    /// Successful render plus EGL swap durations, in fixed buckets: 1..30 ms,
    /// then <=33,333 microseconds, then overflow. No per-frame allocations.
    pub frame_duration_buckets: [u64; 32],
    pub frame_duration_total_micros: u64,
    pub frame_duration_max_micros: u64,
    /// Coalesced result-ready hints consumed for the currently selected Scene
    /// renderer. These do not count rendered frames or result/packet takes.
    pub scene_worker_dispatches: u64,
    /// Publication to engine notification dispatch; excludes subsequent FPS
    /// wait, result consumption, uploads, drawing and swap. No per-event samples.
    pub scene_worker_dispatch_total_micros: u64,
    pub scene_worker_dispatch_max_micros: u64,
}

impl EngineRenderMetrics {
    pub(crate) fn record_scene_worker_dispatch(&mut self, latency: std::time::Duration) {
        let micros = u64::try_from(latency.as_micros()).unwrap_or(u64::MAX);
        self.scene_worker_dispatches = self.scene_worker_dispatches.saturating_add(1);
        self.scene_worker_dispatch_total_micros = self
            .scene_worker_dispatch_total_micros
            .saturating_add(micros);
        self.scene_worker_dispatch_max_micros = self.scene_worker_dispatch_max_micros.max(micros);
    }

    pub(crate) fn record_frame_duration(&mut self, duration: std::time::Duration) {
        let micros = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);
        let bucket = if micros <= 30_000 {
            micros.div_ceil(1_000).saturating_sub(1) as usize
        } else if micros <= 33_333 {
            30
        } else {
            31
        };
        self.frame_duration_buckets[bucket] = self.frame_duration_buckets[bucket].saturating_add(1);
        self.frame_duration_total_micros = self.frame_duration_total_micros.saturating_add(micros);
        self.frame_duration_max_micros = self.frame_duration_max_micros.max(micros);
    }
}

#[cfg(test)]
mod metrics_tests {
    use super::EngineRenderMetrics;
    use std::time::Duration;

    #[test]
    fn worker_dispatch_measurements_are_bounded_and_do_not_count_draws() {
        let mut metrics = EngineRenderMetrics {
            scene_worker_dispatches: u64::MAX - 1,
            scene_worker_dispatch_total_micros: u64::MAX - 1,
            ..Default::default()
        };
        metrics.record_scene_worker_dispatch(Duration::from_micros(7));
        metrics.record_scene_worker_dispatch(Duration::from_micros(2));
        assert_eq!(metrics.scene_worker_dispatches, u64::MAX);
        assert_eq!(metrics.scene_worker_dispatch_total_micros, u64::MAX);
        assert_eq!(metrics.scene_worker_dispatch_max_micros, 7);
        assert_eq!(
            (metrics.render_attempts, metrics.draws, metrics.swaps),
            (0, 0, 0)
        );
    }

    #[test]
    fn bounded_frame_histogram_distinguishes_the_30_fps_budget() {
        let mut metrics = EngineRenderMetrics::default();
        for micros in [0, 1_000, 1_001, 30_000, 30_001, 33_333, 33_334] {
            metrics.record_frame_duration(Duration::from_micros(micros));
        }
        assert_eq!(metrics.frame_duration_buckets[0], 2);
        assert_eq!(metrics.frame_duration_buckets[1], 1);
        assert_eq!(metrics.frame_duration_buckets[29], 1);
        assert_eq!(metrics.frame_duration_buckets[30], 2);
        assert_eq!(metrics.frame_duration_buckets[31], 1);
        assert_eq!(metrics.frame_duration_buckets.iter().sum::<u64>(), 7);
        assert_eq!(metrics.frame_duration_max_micros, 33_334);
    }
}
