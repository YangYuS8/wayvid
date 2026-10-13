//! Shared, output-monitor audio analysis. Capture and FFT never run on the render thread.
//!
//! The Scene interface exposes independently measured 16/32/64 bass-to-treble
//! values per channel in `0..=1`, with a 64-band average for SceneScript.
//! Wallpaper Engine does not document its frequency boundaries or normalization;
//! this implements a Hann-windowed spectrum with logarithmic bands, a -60 dB
//! floor, and bounded attack/release rather than claiming numerical equivalence.

use anyhow::{Context as _, Result, anyhow, ensure};
use libpulse_binding as pulse;
use pulse::callbacks::ListResult;
use pulse::context::{Context, FlagSet as ContextFlags, State as ContextState};
use pulse::mainloop::standard::Mainloop;
use pulse::sample::{Format, Spec};
use pulse::stream::{FlagSet as StreamFlags, PeekResult, State as StreamState, Stream};
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const BANDS: usize = 16;
const RATE: usize = 48_000;
const FFT_SIZE: usize = 4096;
const HOP: usize = 512;
const PCM_FRAME_BYTES: usize = 8;
const MAX_CAPTURE_BYTES: usize = 64 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const STALE_AFTER: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpectrumSnapshot {
    pub(crate) left: [f32; BANDS],
    pub(crate) right: [f32; BANDS],
    /// Independently measured FFT bands for SceneScript; never upsample 16 bands.
    pub(crate) script_average: [f32; 64],
    pub(crate) script_left: [f32; 64],
    pub(crate) script_right: [f32; 64],
    pub(crate) left32: [f32; 32],
    pub(crate) right32: [f32; 32],
}

impl Default for SpectrumSnapshot {
    fn default() -> Self {
        Self {
            left: [0.0; BANDS],
            right: [0.0; BANDS],
            script_average: [0.0; 64],
            script_left: [0.0; 64],
            script_right: [0.0; 64],
            left32: [0.0; 32],
            right32: [0.0; 32],
        }
    }
}

struct AtomicSnapshot {
    generation: AtomicU64,
    left: [AtomicU32; BANDS],
    right: [AtomicU32; BANDS],
    script_average: [AtomicU32; 64],
    script_left: [AtomicU32; 64],
    script_right: [AtomicU32; 64],
    left32: [AtomicU32; 32],
    right32: [AtomicU32; 32],
}

impl Default for AtomicSnapshot {
    fn default() -> Self {
        Self {
            generation: AtomicU64::new(0),
            left: std::array::from_fn(|_| AtomicU32::new(0)),
            right: std::array::from_fn(|_| AtomicU32::new(0)),
            script_average: std::array::from_fn(|_| AtomicU32::new(0)),
            script_left: std::array::from_fn(|_| AtomicU32::new(0)),
            script_right: std::array::from_fn(|_| AtomicU32::new(0)),
            left32: std::array::from_fn(|_| AtomicU32::new(0)),
            right32: std::array::from_fn(|_| AtomicU32::new(0)),
        }
    }
}

impl AtomicSnapshot {
    // Exactly one capture worker writes. Readers make at most three attempts;
    // an interrupted writer must never stall rendering.
    fn publish(&self, value: SpectrumSnapshot) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        for (destination, value) in self.left.iter().zip(value.left) {
            destination.store(value.to_bits(), Ordering::SeqCst);
        }
        for (destination, value) in self.right.iter().zip(value.right) {
            destination.store(value.to_bits(), Ordering::SeqCst);
        }
        for (destination, value) in self.script_average.iter().zip(value.script_average) {
            destination.store(value.to_bits(), Ordering::SeqCst);
        }
        for (destination, value) in self.script_left.iter().zip(value.script_left) {
            destination.store(value.to_bits(), Ordering::SeqCst);
        }
        for (destination, value) in self.script_right.iter().zip(value.script_right) {
            destination.store(value.to_bits(), Ordering::SeqCst);
        }
        for (destination, value) in self.left32.iter().zip(value.left32) {
            destination.store(value.to_bits(), Ordering::SeqCst);
        }
        for (destination, value) in self.right32.iter().zip(value.right32) {
            destination.store(value.to_bits(), Ordering::SeqCst);
        }
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    fn read(&self) -> SpectrumSnapshot {
        for _ in 0..3 {
            let generation = self.generation.load(Ordering::SeqCst);
            if generation & 1 != 0 {
                continue;
            }
            let value = SpectrumSnapshot {
                left: std::array::from_fn(|i| f32::from_bits(self.left[i].load(Ordering::SeqCst))),
                right: std::array::from_fn(|i| {
                    f32::from_bits(self.right[i].load(Ordering::SeqCst))
                }),
                script_average: std::array::from_fn(|i| {
                    f32::from_bits(self.script_average[i].load(Ordering::SeqCst))
                }),
                script_left: std::array::from_fn(|i| {
                    f32::from_bits(self.script_left[i].load(Ordering::SeqCst))
                }),
                script_right: std::array::from_fn(|i| {
                    f32::from_bits(self.script_right[i].load(Ordering::SeqCst))
                }),
                left32: std::array::from_fn(|i| {
                    f32::from_bits(self.left32[i].load(Ordering::SeqCst))
                }),
                right32: std::array::from_fn(|i| {
                    f32::from_bits(self.right32[i].load(Ordering::SeqCst))
                }),
            };
            if generation == self.generation.load(Ordering::SeqCst) {
                return value;
            }
        }
        SpectrumSnapshot::default()
    }
}

#[derive(Default)]
struct CaptureState {
    stop: AtomicBool,
    active: AtomicUsize,
    connected: AtomicBool,
    snapshot: AtomicSnapshot,
}

impl CaptureState {
    fn running(&self) -> bool {
        !self.stop.load(Ordering::Acquire) && self.active.load(Ordering::Acquire) > 0
    }
}

struct CaptureWorker {
    state: Arc<CaptureState>,
    thread: Option<JoinHandle<()>>,
}

impl CaptureWorker {
    fn new(monitor: Option<String>) -> Result<Arc<Self>> {
        let state = Arc::new(CaptureState::default());
        let worker_state = Arc::clone(&state);
        let thread = thread::Builder::new()
            .name("lwe-scene-spectrum".into())
            .spawn(move || capture_worker(worker_state, monitor))
            .context("Start Scene audio analysis worker")?;
        Ok(Arc::new(Self {
            state,
            thread: Some(thread),
        }))
    }

    fn wake(&self) {
        if let Some(thread) = &self.thread {
            thread.thread().unpark();
        }
    }
}

impl Drop for CaptureWorker {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Release);
        self.wake();
        if let Some(thread) = self.thread.take() {
            // The worker exclusively uses nonblocking Pulse iterations. No
            // pa_simple_read, child process, or server operation can hold drop.
            if thread.join().is_err() {
                tracing::warn!("Scene audio analysis worker stopped unexpectedly");
            }
        }
    }
}

static SHARED_CAPTURE: Mutex<Weak<CaptureWorker>> = Mutex::new(Weak::new());

/// A paused-by-default analysis consumer. All Scenes share one monitor stream.
/// Activate only after the candidate is finalized; pause during rollback/clear.
pub(crate) struct Spectrum {
    worker: Arc<CaptureWorker>,
    paused: AtomicBool,
    pause_update: Mutex<()>,
}

impl Spectrum {
    pub(crate) fn new() -> Result<Self> {
        let mut shared = SHARED_CAPTURE
            .lock()
            .map_err(|_| anyhow!("Scene audio analysis ownership is unavailable"))?;
        let worker = if let Some(worker) = shared.upgrade() {
            worker
        } else {
            let worker = CaptureWorker::new(None)?;
            *shared = Arc::downgrade(&worker);
            worker
        };
        Ok(Self {
            worker,
            paused: AtomicBool::new(true),
            pause_update: Mutex::new(()),
        })
    }

    pub(crate) fn set_paused(&self, paused: bool) {
        // Serialize only this handle's two-atomic state transition. Snapshots
        // remain lock-free; distinct Scene consumers never contend here.
        let _update = self
            .pause_update
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if self.paused.swap(paused, Ordering::AcqRel) == paused {
            return;
        }
        if paused {
            self.worker.state.active.fetch_sub(1, Ordering::AcqRel);
        } else {
            self.worker.state.active.fetch_add(1, Ordering::AcqRel);
        }
        self.worker.wake();
    }

    pub(crate) fn snapshot(&self) -> SpectrumSnapshot {
        if self.paused.load(Ordering::Acquire) {
            SpectrumSnapshot::default()
        } else {
            self.worker.state.snapshot.read()
        }
    }

    #[cfg(test)]
    pub(in crate::scene) fn private_monitor(monitor: String) -> Self {
        Self {
            worker: CaptureWorker::new(Some(monitor)).unwrap(),
            paused: AtomicBool::new(true),
            pause_update: Mutex::new(()),
        }
    }

    #[cfg(test)]
    pub(in crate::scene) fn capture_probe(&self) -> CaptureProbe {
        CaptureProbe {
            worker: Arc::downgrade(&self.worker),
            state: Arc::clone(&self.worker.state),
        }
    }
}

#[cfg(test)]
pub(in crate::scene) struct CaptureProbe {
    worker: Weak<CaptureWorker>,
    state: Arc<CaptureState>,
}

#[cfg(test)]
impl CaptureProbe {
    pub(in crate::scene) fn connected(&self) -> bool {
        self.state.connected.load(Ordering::Acquire)
    }

    pub(in crate::scene) fn released(&self) -> bool {
        self.worker.upgrade().is_none()
    }
}

impl Drop for Spectrum {
    fn drop(&mut self) {
        self.set_paused(true);
    }
}

struct PulseCapture {
    // Disconnect explicitly before dropping these, in stream/context/mainloop order.
    stream: Stream,
    context: Context,
    mainloop: Mainloop,
    route_changed: Rc<Cell<bool>>,
}

impl PulseCapture {
    fn connect(state: &CaptureState, monitor_override: Option<&str>) -> Result<Self> {
        let mut mainloop = Mainloop::new().context("Create audio analysis mainloop")?;
        let mut context = Context::new(&mainloop, "LWE Scene audio analysis")
            .context("Create audio analysis connection")?;
        // Analysis must not start an audio server or select the default input.
        context.connect(None, ContextFlags::NOAUTOSPAWN, None)?;
        wait_for(&mut mainloop, state, || match context.get_state() {
            ContextState::Ready => Ok(true),
            ContextState::Failed | ContextState::Terminated => {
                Err(anyhow!("Audio analysis server is unavailable"))
            }
            _ => Ok(false),
        })?;

        let monitor = if let Some(monitor) = monitor_override {
            monitor.to_owned()
        } else {
            let selected = Rc::new(RefCell::new(None));
            let callback_result = Rc::clone(&selected);
            let mut operation = context.introspect().get_server_info(move |info| {
                *callback_result.borrow_mut() =
                    Some(info.default_sink_name.as_ref().map(|name| name.to_string()));
            });
            let result = wait_for(&mut mainloop, state, || Ok(selected.borrow().is_some()));
            if result.is_err() {
                operation.cancel();
            }
            result?;
            let sink = selected
                .borrow_mut()
                .take()
                .flatten()
                .context("Audio analysis has no output sink")?;
            let selected = Rc::new(RefCell::new(None));
            let callback_result = Rc::clone(&selected);
            let mut operation = context
                .introspect()
                .get_sink_info_by_name(&sink, move |info| match info {
                    ListResult::Item(info) => {
                        *callback_result.borrow_mut() = Some(
                            info.monitor_source_name
                                .as_ref()
                                .map(|name| name.to_string()),
                        );
                    }
                    ListResult::End | ListResult::Error => {
                        if callback_result.borrow().is_none() {
                            *callback_result.borrow_mut() = Some(None);
                        }
                    }
                });
            let result = wait_for(&mut mainloop, state, || Ok(selected.borrow().is_some()));
            if result.is_err() {
                operation.cancel();
            }
            result?;
            selected
                .borrow_mut()
                .take()
                .flatten()
                .context("Audio analysis output has no monitor")?
        };

        let route_changed = Rc::new(Cell::new(false));
        if monitor_override.is_none() {
            let changed = Rc::clone(&route_changed);
            context.set_subscribe_callback(Some(Box::new(move |facility, operation, _| {
                use pulse::context::subscribe::{Facility, Operation};
                if facility == Some(Facility::Server)
                    || (facility == Some(Facility::Sink)
                        && matches!(operation, Some(Operation::New | Operation::Removed)))
                {
                    changed.set(true);
                }
            })));
            context.subscribe(
                pulse::context::subscribe::InterestMaskSet::SERVER
                    | pulse::context::subscribe::InterestMaskSet::SINK,
                |_| {},
            );
        }
        let spec = Spec {
            format: Format::F32le,
            channels: 2,
            rate: RATE as u32,
        };
        let mut channels = pulse::channelmap::Map::default();
        channels.init_stereo();
        let mut stream = Stream::new(
            &mut context,
            "LWE Scene output spectrum",
            &spec,
            Some(&channels),
        )
        .context("Create audio analysis stream")?;
        let attributes = pulse::def::BufferAttr {
            maxlength: MAX_CAPTURE_BYTES as u32,
            fragsize: (RATE / 50 * PCM_FRAME_BYTES) as u32,
            tlength: u32::MAX,
            prebuf: u32::MAX,
            minreq: u32::MAX,
        };
        // DONT_MOVE prevents policy fallback to a microphone. Not requesting
        // ADJUST_LATENCY also leaves the user's sink/source latency policy alone.
        stream.connect_record(Some(&monitor), Some(&attributes), StreamFlags::DONT_MOVE)?;
        wait_for(&mut mainloop, state, || match stream.get_state() {
            StreamState::Ready => Ok(true),
            StreamState::Failed | StreamState::Terminated => {
                Err(anyhow!("Audio analysis monitor is unavailable"))
            }
            _ => Ok(false),
        })?;
        Ok(Self {
            stream,
            context,
            mainloop,
            route_changed,
        })
    }

    fn collect(&mut self, analyzer: &mut Analyzer, state: &CaptureState) -> Result<bool> {
        for _ in 0..8 {
            ensure!(
                self.mainloop.iterate(false).is_success(),
                "Audio analysis dispatch failed"
            );
        }
        ensure!(
            self.context.get_state() == ContextState::Ready
                && self.stream.get_state() == StreamState::Ready
                && !self.route_changed.get(),
            "Audio analysis connection changed"
        );
        let mut received = false;
        // Finite work even if the audio server delivers faster than real time.
        for _ in 0..8 {
            if !state.running() {
                break;
            }
            match self.stream.peek()? {
                PeekResult::Empty => break,
                PeekResult::Hole(bytes) => {
                    ensure!(
                        bytes <= MAX_CAPTURE_BYTES,
                        "Audio analysis gap exceeds buffer limit"
                    );
                    for _ in 0..bytes / PCM_FRAME_BYTES {
                        if let Some(snapshot) = analyzer.push(0.0, 0.0) {
                            state.snapshot.publish(snapshot);
                        }
                    }
                }
                PeekResult::Data(bytes) => {
                    ensure!(
                        bytes.len() <= MAX_CAPTURE_BYTES && bytes.len() % PCM_FRAME_BYTES == 0,
                        "Audio analysis frame buffer is invalid"
                    );
                    for frame in bytes.as_chunks::<PCM_FRAME_BYTES>().0 {
                        let left = f32::from_le_bytes(frame[..4].try_into().unwrap());
                        let right = f32::from_le_bytes(frame[4..].try_into().unwrap());
                        if let Some(snapshot) = analyzer.push(left, right) {
                            state.snapshot.publish(snapshot);
                        }
                    }
                }
            }
            self.stream.discard()?;
            received = true;
        }
        Ok(received)
    }
}

impl Drop for PulseCapture {
    fn drop(&mut self) {
        let _ = self.stream.disconnect();
        self.context.set_subscribe_callback(None);
        self.context.disconnect();
    }
}

fn wait_for(
    mainloop: &mut Mainloop,
    state: &CaptureState,
    mut ready: impl FnMut() -> Result<bool>,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        ensure!(state.running(), "Audio analysis connection cancelled");
        if ready()? {
            return Ok(());
        }
        ensure!(
            Instant::now() < deadline,
            "Audio analysis connection timed out"
        );
        ensure!(
            mainloop.iterate(false).is_success(),
            "Audio analysis dispatch failed"
        );
        thread::park_timeout(POLL_INTERVAL);
    }
}

fn capture_worker(state: Arc<CaptureState>, monitor: Option<String>) {
    let mut analyzer = Analyzer::new();
    while !state.stop.load(Ordering::Acquire) {
        if !state.running() {
            state.snapshot.publish(SpectrumSnapshot::default());
            thread::park_timeout(Duration::from_millis(100));
            continue;
        }
        let capture = PulseCapture::connect(&state, monitor.as_deref());
        if let Ok(mut capture) = capture {
            analyzer.reset();
            state.connected.store(true, Ordering::Release);
            let mut last_pcm = Instant::now();
            while state.running() {
                match capture.collect(&mut analyzer, &state) {
                    Ok(true) => last_pcm = Instant::now(),
                    Ok(false) => {
                        if last_pcm.elapsed() > STALE_AFTER {
                            analyzer.reset();
                            state.snapshot.publish(SpectrumSnapshot::default());
                        }
                    }
                    Err(_) => break,
                }
                thread::park_timeout(POLL_INTERVAL);
            }
        }
        state.connected.store(false, Ordering::Release);
        state.snapshot.publish(SpectrumSnapshot::default());
        // Missing servers, device removal, and silence cannot fail playback.
        // Retry in short cancellable slices; never busy-loop on an absent server.
        for _ in 0..100 {
            if !state.running() {
                break;
            }
            thread::park_timeout(POLL_INTERVAL);
        }
    }
}

struct Analyzer {
    fft: Arc<dyn Fft<f32>>,
    ring: [[f32; FFT_SIZE]; 2],
    window: [f32; FFT_SIZE],
    buffer: Vec<Complex<f32>>,
    scratch: Vec<Complex<f32>>,
    band_bins: [(usize, usize); BANDS],
    script_band_bins: [(usize, usize); 64],
    band_bins32: [(usize, usize); 32],
    scale: f32,
    write: usize,
    filled: usize,
    hop: usize,
    smoothed: SpectrumSnapshot,
    script_left: [f32; 64],
    script_right: [f32; 64],
}

fn frequency_bins<const N: usize>() -> [(usize, usize); N] {
    let edge = |i: usize| {
        if i == 0 {
            20.0
        } else {
            32.0 * (20_000.0_f32 / 32.0).powf(i as f32 / N as f32)
        }
    };
    std::array::from_fn(|i| {
        let first = (edge(i) * FFT_SIZE as f32 / RATE as f32).ceil() as usize;
        let end = (edge(i + 1) * FFT_SIZE as f32 / RATE as f32).ceil() as usize;
        (first.max(1), end.max(first + 1).min(FFT_SIZE / 2))
    })
}

impl Analyzer {
    fn new() -> Self {
        let fft = FftPlanner::new().plan_fft_forward(FFT_SIZE);
        let window: [f32; FFT_SIZE] = std::array::from_fn(|i| {
            0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / (FFT_SIZE - 1) as f32).cos()
        });
        // Coherent amplitude normalization: a full-scale sine centered on an
        // FFT bin has magnitude one. Frequency zero is deliberately excluded.
        let scale = 2.0 / window.iter().sum::<f32>();
        let band_bins = frequency_bins();
        Self {
            scratch: vec![Complex::default(); fft.get_inplace_scratch_len()],
            fft,
            ring: [[0.0; FFT_SIZE]; 2],
            window,
            buffer: vec![Complex::default(); FFT_SIZE],
            band_bins,
            script_band_bins: frequency_bins(),
            band_bins32: frequency_bins(),
            scale,
            write: 0,
            filled: 0,
            hop: 0,
            smoothed: SpectrumSnapshot::default(),
            script_left: [0.0; 64],
            script_right: [0.0; 64],
        }
    }

    fn reset(&mut self) {
        self.ring.fill([0.0; FFT_SIZE]);
        self.write = 0;
        self.filled = 0;
        self.hop = 0;
        self.smoothed = SpectrumSnapshot::default();
        self.script_left = [0.0; 64];
        self.script_right = [0.0; 64];
    }

    fn push(&mut self, left: f32, right: f32) -> Option<SpectrumSnapshot> {
        for (channel, value) in [left, right].into_iter().enumerate() {
            self.ring[channel][self.write] = if value.is_finite() {
                value.clamp(-1.0, 1.0)
            } else {
                0.0
            };
        }
        self.write = (self.write + 1) % FFT_SIZE;
        self.filled = (self.filled + 1).min(FFT_SIZE);
        self.hop += 1;
        if self.filled < FFT_SIZE || self.hop < HOP {
            return None;
        }
        self.hop = 0;
        self.analyze_channel(0);
        let left = self.smooth_bands(&self.band_bins, self.smoothed.left);
        self.script_left = self.smooth_bands(&self.script_band_bins, self.script_left);
        let left32 = self.smooth_bands(&self.band_bins32, self.smoothed.left32);
        self.analyze_channel(1);
        let right = self.smooth_bands(&self.band_bins, self.smoothed.right);
        self.script_right = self.smooth_bands(&self.script_band_bins, self.script_right);
        let right32 = self.smooth_bands(&self.band_bins32, self.smoothed.right32);
        self.smoothed = SpectrumSnapshot {
            left,
            right,
            script_average: std::array::from_fn(|i| {
                (self.script_left[i] + self.script_right[i]) * 0.5
            }),
            script_left: self.script_left,
            script_right: self.script_right,
            left32,
            right32,
        };
        Some(self.smoothed)
    }

    fn analyze_channel(&mut self, channel: usize) {
        for (i, value) in self.buffer.iter_mut().enumerate() {
            *value = Complex::new(
                self.ring[channel][(self.write + i) % FFT_SIZE] * self.window[i],
                0.0,
            );
        }
        self.fft
            .process_with_scratch(&mut self.buffer, &mut self.scratch);
    }

    fn smooth_bands<const N: usize>(
        &self,
        bins: &[(usize, usize); N],
        previous: [f32; N],
    ) -> [f32; N] {
        std::array::from_fn(|band| {
            let (first, end) = bins[band];
            let magnitude = self.buffer[first..end]
                .iter()
                .map(|value| value.norm() * self.scale)
                .fold(0.0_f32, f32::max);
            let target = if magnitude <= 0.001 {
                0.0
            } else {
                ((20.0 * magnitude.log10() + 60.0) / 60.0).clamp(0.0, 1.0)
            };
            let time = if target > previous[band] { 0.025 } else { 0.2 };
            let amount = 1.0 - (-(HOP as f32 / RATE as f32) / time).exp();
            let value = previous[band] + (target - previous[band]) * amount;
            if value < 0.0001 {
                0.0
            } else {
                value.clamp(0.0, 1.0)
            }
        })
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};

    fn tone(analyzer: &mut Analyzer, left_hz: f32, right_hz: f32, gain: f32) -> SpectrumSnapshot {
        let mut result = SpectrumSnapshot::default();
        for i in 0..RATE {
            let time = i as f32 / RATE as f32;
            if let Some(value) = analyzer.push(
                gain * (std::f32::consts::TAU * left_hz * time).sin(),
                gain * (std::f32::consts::TAU * right_hz * time).sin(),
            ) {
                result = value;
            }
        }
        result
    }

    fn strongest(values: &[f32; BANDS]) -> usize {
        values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap()
            .0
    }

    #[test]
    fn stereo_bass_and_treble_remain_independent() {
        let mut analyzer = Analyzer::new();
        let value = tone(&mut analyzer, 46.875, 6000.0, 0.5);
        assert_eq!(strongest(&value.left), 0);
        assert!(strongest(&value.right) > 10);
        assert!(value.left[0] > 0.8);
        assert!(value.right[0] < 0.01);
        assert!(value.left[strongest(&value.right)] < 0.01);
    }

    #[test]
    fn script_bands_resolve_tones_inside_one_shader_band() {
        let mut first = Analyzer::new();
        let mut second = Analyzer::new();
        let low = tone(&mut first, 1000.0, 1000.0, 0.5);
        let high = tone(&mut second, 1125.0, 1125.0, 0.5);
        assert_eq!(strongest(&low.left), strongest(&high.left));
        let peak = |bands: &[f32; 64]| {
            bands
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .unwrap()
                .0
        };
        assert_ne!(peak(&low.script_average), peak(&high.script_average));
        assert!(low.script_average.iter().any(|value| *value > 0.8));
        first.reset();
        assert_eq!(tone(&mut first, 0.0, 0.0, 0.0).script_average, [0.0; 64]);
    }

    #[test]
    fn all_band_resolutions_preserve_stereo_and_atomic_snapshot_identity() {
        let mut analyzer = Analyzer::new();
        let value = tone(&mut analyzer, 46.875, 6000.0, 0.5);
        let peak = |bands: &[f32]| {
            bands
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .unwrap()
                .0
        };
        assert!(peak(&value.script_left) < 8);
        assert!(peak(&value.left32) < 4);
        assert!(peak(&value.script_right) > 40);
        assert!(peak(&value.right32) > 20);
        // The narrower 32/64 first band contains the Hann side lobe of this
        // bin-centered tone, not its peak. Check the measured peak and stereo
        // rejection there; a gain-0.5 centered tone maps above 0.85 at -60dB.
        let left64 = peak(&value.script_left);
        let right64 = peak(&value.script_right);
        let left32 = peak(&value.left32);
        let right32 = peak(&value.right32);
        assert!(value.script_left[left64] > 0.85 && value.script_right[left64] < 0.01);
        assert!(value.script_right[right64] > 0.85 && value.script_left[right64] < 0.01);
        assert!(value.left32[left32] > 0.85 && value.right32[left32] < 0.01);
        assert!(value.right32[right32] > 0.85 && value.left32[right32] < 0.01);
        for index in 0..64 {
            assert_eq!(
                value.script_average[index],
                (value.script_left[index] + value.script_right[index]) * 0.5
            );
        }
        let atomic = AtomicSnapshot::default();
        atomic.publish(value);
        assert_eq!(atomic.read(), value);
        atomic.publish(SpectrumSnapshot::default());
        assert_eq!(atomic.read(), SpectrumSnapshot::default());
        atomic.generation.store(1, Ordering::SeqCst);
        assert_eq!(atomic.read(), SpectrumSnapshot::default());
    }

    #[test]
    fn quiet_nonfinite_and_clipping_are_finite_and_bounded() {
        let mut analyzer = Analyzer::new();
        assert_eq!(
            tone(&mut analyzer, 1500.0, 1500.0, 0.00001),
            SpectrumSnapshot::default()
        );
        for _ in 0..RATE {
            if let Some(value) = analyzer.push(f32::NAN, f32::INFINITY) {
                assert_eq!(value, SpectrumSnapshot::default());
            }
        }
        let value = tone(&mut analyzer, 1500.0, 3000.0, 10.0);
        assert!(
            value
                .left
                .into_iter()
                .chain(value.right)
                .chain(value.left32)
                .chain(value.right32)
                .chain(value.script_left)
                .chain(value.script_right)
                .chain(value.script_average)
                .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
        );
        assert!(value.left[strongest(&value.left)] > 0.9);
    }

    #[test]
    fn silence_releases_and_reset_removes_the_old_window() {
        let mut analyzer = Analyzer::new();
        let value = tone(&mut analyzer, 1500.0, 1500.0, 0.5);
        assert!(value.left.iter().any(|value| *value > 0.8));
        let mut value = value;
        for _ in 0..RATE * 3 {
            if let Some(next) = analyzer.push(0.0, 0.0) {
                value = next;
            }
        }
        assert_eq!(value, SpectrumSnapshot::default());
        tone(&mut analyzer, 1500.0, 1500.0, 0.5);
        analyzer.reset();
        assert_eq!(
            tone(&mut analyzer, 0.0, 0.0, 0.0),
            SpectrumSnapshot::default()
        );
    }

    #[test]
    fn shared_capture_counts_consumers_and_paused_snapshots_are_immediate() {
        let shared_first = Spectrum::new().unwrap();
        let shared_second = Spectrum::new().unwrap();
        assert!(Arc::ptr_eq(&shared_first.worker, &shared_second.worker));
        drop((shared_first, shared_second));
        // Count and lifetime assertions use their own worker so concurrent
        // renderer tests cannot add a legitimate global consumer between reads.
        let worker = Arc::new(CaptureWorker {
            state: Arc::new(CaptureState::default()),
            thread: None,
        });
        let first = Spectrum {
            worker: Arc::clone(&worker),
            paused: AtomicBool::new(true),
            pause_update: Mutex::new(()),
        };
        let second = Spectrum {
            worker,
            paused: AtomicBool::new(true),
            pause_update: Mutex::new(()),
        };
        assert_eq!(first.worker.state.active.load(Ordering::Acquire), 0);
        first.set_paused(false);
        first.set_paused(false);
        second.set_paused(false);
        assert_eq!(first.worker.state.active.load(Ordering::Acquire), 2);
        first.set_paused(true);
        assert_eq!(first.snapshot(), SpectrumSnapshot::default());
        assert_eq!(first.worker.state.active.load(Ordering::Acquire), 1);
        drop(second);
        assert_eq!(first.worker.state.active.load(Ordering::Acquire), 0);
        let first = Arc::new(first);
        let callers: Vec<_> = (0..4)
            .map(|_| {
                let first = Arc::clone(&first);
                thread::spawn(move || {
                    for _ in 0..100 {
                        first.set_paused(false);
                        first.set_paused(true);
                    }
                })
            })
            .collect();
        for caller in callers {
            caller.join().unwrap();
        }
        first.set_paused(true);
        assert_eq!(first.worker.state.active.load(Ordering::Acquire), 0);
        let weak = Arc::downgrade(&first.worker);
        drop(first);
        assert!(weak.upgrade().is_none());
    }

    pub(in crate::scene) fn pactl(args: &[&str]) -> String {
        let result = Command::new("pactl")
            .args(args)
            .output()
            .expect("pactl is required for the opt-in spectrum test");
        assert!(
            result.status.success(),
            "Private spectrum sink command failed"
        );
        String::from_utf8(result.stdout).unwrap().trim().to_owned()
    }

    pub(in crate::scene) struct OwnedSink {
        pub(in crate::scene) name: String,
        module: String,
    }

    impl OwnedSink {
        pub(in crate::scene) fn new() -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let name = format!("lwe_spectrum_acceptance_{}_{unique}", std::process::id());
            let module = pactl(&[
                "load-module",
                "module-null-sink",
                &format!("sink_name={name}"),
                "rate=48000",
                "channels=2",
                "sink_properties=device.description=LWE_Spectrum_Acceptance",
            ]);
            assert!(module.parse::<u32>().is_ok());
            Self { name, module }
        }
    }

    impl Drop for OwnedSink {
        fn drop(&mut self) {
            let result = Command::new("pactl")
                .args(["unload-module", &self.module])
                .output();
            assert!(result.is_ok_and(|result| result.status.success()));
        }
    }

    pub(in crate::scene) struct OwnedPlayback {
        child: Child,
        path: PathBuf,
    }

    impl OwnedPlayback {
        pub(in crate::scene) fn new(sink: &OwnedSink) -> Self {
            let path = std::env::temp_dir().join(format!("{}.wav", sink.name));
            let frames = RATE * 12;
            let payload_bytes = (frames * 4) as u32;
            let mut wave = Vec::with_capacity(payload_bytes as usize + 44);
            wave.extend_from_slice(b"RIFF");
            wave.extend_from_slice(&(payload_bytes + 36).to_le_bytes());
            wave.extend_from_slice(b"WAVEfmt ");
            wave.extend_from_slice(&16_u32.to_le_bytes());
            wave.extend_from_slice(&1_u16.to_le_bytes());
            wave.extend_from_slice(&2_u16.to_le_bytes());
            wave.extend_from_slice(&(RATE as u32).to_le_bytes());
            wave.extend_from_slice(&(RATE as u32 * 4).to_le_bytes());
            wave.extend_from_slice(&4_u16.to_le_bytes());
            wave.extend_from_slice(&16_u16.to_le_bytes());
            wave.extend_from_slice(b"data");
            wave.extend_from_slice(&payload_bytes.to_le_bytes());
            for frame in 0..frames {
                let time = frame as f32 / RATE as f32;
                for frequency in [46.875, 6000.0] {
                    let sample = ((std::f32::consts::TAU * frequency * time).sin()
                        * 0.5
                        * i16::MAX as f32) as i16;
                    wave.extend_from_slice(&sample.to_le_bytes());
                }
            }
            fs::write(&path, wave).unwrap();
            let child = Command::new("paplay")
                .arg(format!("--device={}", sink.name))
                .arg(&path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("paplay is required for the opt-in spectrum test");
            Self { child, path }
        }
    }

    impl Drop for OwnedPlayback {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = fs::remove_file(&self.path);
        }
    }

    fn eventually(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(4);
        while !condition() {
            assert!(
                Instant::now() < deadline,
                "Audio spectrum acceptance timed out"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    pub(in crate::scene) fn owned_capture_sources() -> Vec<u32> {
        let value: serde_json::Value =
            serde_json::from_str(&pactl(&["--format=json", "list", "source-outputs"])).unwrap();
        value
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|stream| {
                let properties = &stream["properties"];
                (properties["application.process.id"].as_str()
                    == Some(&std::process::id().to_string())
                    && properties["media.name"].as_str() == Some("LWE Scene output spectrum"))
                .then(|| stream["source"].as_u64().unwrap() as u32)
            })
            .collect()
    }

    #[test]
    #[ignore = "requires LWE_REAL_DESKTOP_TESTS=1, PulseAudio/PipeWire, pactl and paplay"]
    fn private_monitor_observes_stereo_pause_resume_silence_and_cleanup() {
        assert_eq!(std::env::var("LWE_REAL_DESKTOP_TESTS").as_deref(), Ok("1"));
        let default_sink = pactl(&["get-default-sink"]);
        let default_source = pactl(&["get-default-source"]);
        assert!(owned_capture_sources().is_empty());
        let sink = OwnedSink::new();
        let sources: serde_json::Value =
            serde_json::from_str(&pactl(&["--format=json", "list", "sources"])).unwrap();
        let monitor_index = sources
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["name"].as_str() == Some(&format!("{}.monitor", sink.name)))
            .unwrap()["index"]
            .as_u64()
            .unwrap() as u32;
        let spectrum = Spectrum::private_monitor(format!("{}.monitor", sink.name));
        let state = Arc::clone(&spectrum.worker.state);
        let weak = Arc::downgrade(&spectrum.worker);
        assert_eq!(spectrum.snapshot(), SpectrumSnapshot::default());
        assert!(!state.connected.load(Ordering::Acquire));
        spectrum.set_paused(false);
        eventually(|| state.connected.load(Ordering::Acquire));
        assert_eq!(owned_capture_sources(), vec![monitor_index]);
        let playback = OwnedPlayback::new(&sink);
        eventually(|| {
            let value = spectrum.snapshot();
            value.left[0] > 0.75 && value.right.iter().any(|value| *value > 0.75)
        });
        let active = spectrum.snapshot();
        assert_eq!(strongest(&active.left), 0);
        assert!(strongest(&active.right) > 10);
        assert!(active.right[0] < 0.01);
        spectrum.set_paused(true);
        assert_eq!(spectrum.snapshot(), SpectrumSnapshot::default());
        eventually(|| !state.connected.load(Ordering::Acquire));
        eventually(|| owned_capture_sources().is_empty());
        spectrum.set_paused(false);
        eventually(|| spectrum.snapshot().left[0] > 0.75);
        let resumed = spectrum.snapshot();
        assert_eq!(strongest(&resumed.left), 0);
        assert!(strongest(&resumed.right) > 10);
        assert_eq!(owned_capture_sources(), vec![monitor_index]);
        drop(playback);
        eventually(|| {
            spectrum
                .snapshot()
                .left
                .iter()
                .chain(spectrum.snapshot().right.iter())
                .all(|value| *value < 0.01)
        });
        drop(spectrum);
        assert!(weak.upgrade().is_none(), "Capture worker ownership leaked");
        assert!(!state.connected.load(Ordering::Acquire));
        eventually(|| owned_capture_sources().is_empty());
        drop(sink);
        assert_eq!(pactl(&["get-default-sink"]), default_sink);
        assert_eq!(pactl(&["get-default-source"]), default_source);
        eprintln!(
            "scene_spectrum_monitor_acceptance {}",
            serde_json::json!({
                "active_left": active.left,
                "active_right": active.right,
                "resumed_left": resumed.left,
                "resumed_right": resumed.right,
                "active_script_average": active.script_average.to_vec(),
                "resumed_script_average": resumed.script_average.to_vec(),
                "stereo_independent": true,
                "capture_source_was_owned_sink_monitor": true,
                "pause_disconnected_capture": true,
                "resume_reconnected_capture": true,
                "silence_released": true,
                "capture_worker_joined": true,
                "owned_sink_unloaded": true,
                "default_sink_unchanged": true,
                "default_source_unchanged": true,
            })
        );
    }
}
