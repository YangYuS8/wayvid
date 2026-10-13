//! Bounded CPU frame jobs; GL remains on the engine/context thread.

use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;

use anyhow::{Context, Result, ensure};
use lwe_scene::{PixelRect, TimedAtlasRegion, TimedAtlasSource};
use sha2::{Digest, Sha256};

use super::shared_texture::TextureKey;

pub fn scene_texture_memory_budget() -> &'static lwe_scene::AtlasMemoryBudget {
    static BUDGET: OnceLock<lwe_scene::AtlasMemoryBudget> = OnceLock::new();
    BUDGET.get_or_init(|| {
        lwe_scene::AtlasMemoryBudget::new(512 * 1024 * 1024).expect("fixed Scene CPU budget")
    })
}

static BUSY: AtomicBool = AtomicBool::new(false);

struct Permit;
impl Permit {
    fn acquire() -> Option<Self> {
        BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| Self)
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::Release);
    }
}

#[derive(Clone)]
pub(super) struct Source {
    pub index: usize,
    pub fingerprint: [u8; 32],
    pub source: Arc<TimedAtlasSource>,
}

pub(super) struct FrameRegion {
    pub index: usize,
    pub frame: usize,
    pub rect: PixelRect,
    pub pixels: TimedAtlasRegion,
    pub key: TextureKey,
    pub virtual_size: [u32; 2],
}

#[derive(Clone)]
pub(super) struct TextSource {
    pub layer: usize,
    pub index: usize,
    pub prepared: super::text::PreparedText,
    pub value: String,
}
pub(super) struct FrameText {
    pub layer: usize,
    pub index: usize,
    pub value: String,
    pub pixels: super::text::TextFrame,
    pub key: TextureKey,
}

pub(super) struct ReadyFrame {
    pub seq: u64,
    pub time: f64,
    pub regions: Vec<FrameRegion>,
    pub texts: Vec<FrameText>,
    _permit: Permit,
}

struct Job {
    seq: u64,
    time: f64,
    sources: Vec<Source>,
    texts: Vec<TextSource>,
    permit: Permit,
}

/// Each renderer owns a mailbox and a cancellation epoch. The global permit
/// includes queued ready buffers until upload consumes them, bounding producers.
pub(super) struct TimedWorker {
    input: Option<mpsc::SyncSender<Job>>,
    output: mpsc::Receiver<Result<ReadyFrame>>,
    stopped: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    pending: bool,
}

impl TimedWorker {
    pub fn new() -> Result<Self> {
        Self::start(
            #[cfg(test)]
            None,
        )
    }
    #[cfg(test)]
    pub fn held(gate: Arc<AtomicBool>) -> Result<Self> {
        Self::start(Some(gate))
    }
    fn start(#[cfg(test)] gate: Option<Arc<AtomicBool>>) -> Result<Self> {
        let (input, jobs) = mpsc::sync_channel::<Job>(1);
        let (results, output) = mpsc::sync_channel(1);
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&stopped);
        let worker = thread::Builder::new()
            .name("lwe-scene-texture".into())
            .spawn(move || {
                while let Ok(job) = jobs.recv() {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    let result = prepare(job, &stop);
                    #[cfg(test)]
                    while gate
                        .as_ref()
                        .is_some_and(|gate| !gate.load(Ordering::Acquire))
                        && !stop.load(Ordering::Acquire)
                    {
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    if results.send(result).is_err() {
                        break;
                    }
                }
            })
            .context("Cannot start Scene texture worker")?;
        Ok(Self {
            input: Some(input),
            output,
            stopped,
            thread: Some(worker),
            pending: false,
        })
    }

    pub fn request(
        &mut self,
        seq: u64,
        time: f64,
        sources: &[Source],
        texts: Vec<TextSource>,
    ) -> Result<bool> {
        ensure!(time.is_finite() && time >= 0.0, "Invalid timed frame time");
        if self.pending {
            return Ok(false);
        }
        let Some(permit) = Permit::acquire() else {
            return Ok(false);
        };
        self.input
            .as_ref()
            .context("Scene texture worker stopped")?
            .try_send(Job {
                seq,
                time,
                sources: sources.to_vec(),
                texts,
                permit,
            })
            .context("Scene texture worker request failed")?;
        self.pending = true;
        Ok(true)
    }

    pub fn take(&mut self) -> Result<Option<ReadyFrame>> {
        match self.output.try_recv() {
            Ok(result) => {
                self.pending = false;
                result.map(Some)
            }
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => anyhow::bail!("Scene texture worker stopped"),
        }
    }
}

fn prepare(job: Job, stop: &AtomicBool) -> Result<ReadyFrame> {
    let mut regions = Vec::new();
    for source in job.sources {
        ensure!(!stop.load(Ordering::Acquire), "Scene texture job cancelled");
        let frame = source.source.frame_at(job.time)?;
        let rect = source.source.frames()[frame].rect;
        let layout = source.source.frame_neighborhood(frame, 1)?;
        let pixels = source.source.decode_region(layout.rect())?;
        let mut digest = Sha256::new();
        digest.update(b"lwe-timed-region-top-down-linear-clamp-v1");
        digest.update(source.fingerprint);
        digest.update((frame as u64).to_le_bytes());
        for value in [
            layout.rect().x,
            layout.rect().y,
            layout.rect().width,
            layout.rect().height,
        ] {
            digest.update(value.to_le_bytes());
        }
        digest.update(pixels.rgba());
        regions.push(FrameRegion {
            index: source.index,
            frame,
            rect,
            key: TextureKey {
                fingerprint: digest.finalize().into(),
                repeat: false,
            },
            virtual_size: [
                source.source.metadata().storage_width,
                source.source.metadata().storage_height,
            ],
            pixels,
        });
    }
    let mut texts = Vec::new();
    for text in job.texts {
        ensure!(!stop.load(Ordering::Acquire), "Scene texture job cancelled");
        let pixels = text.prepared.rasterize(&text.value)?;
        let key = TextureKey {
            fingerprint: pixels.fingerprint(),
            repeat: false,
        };
        texts.push(FrameText {
            layer: text.layer,
            index: text.index,
            value: text.value,
            pixels,
            key,
        });
    }
    Ok(ReadyFrame {
        seq: job.seq,
        time: job.time,
        regions,
        texts,
        _permit: job.permit,
    })
}

impl Drop for TimedWorker {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        self.input = None;
        // A cancelled decoder keeps its source and capacity leases until it
        // exits. Never join/decompress/hash on the engine thread.
        if let Some(worker) = self.thread.take() {
            let _ = thread::Builder::new()
                .name("lwe-texture-reap".into())
                .spawn(move || {
                    let _ = worker.join();
                });
        }
    }
}
