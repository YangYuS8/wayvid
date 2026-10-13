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

// The sealed encoded source and region determine the pure BC3 output. Keep
// its actual content hash even if a caller supplies a stale prepared identity.
fn region_key(source: &Source, frame: usize, rect: PixelRect) -> TextureKey {
    let mut digest = Sha256::new();
    digest.update(b"lwe-timed-region-sealed-bc3-top-down-linear-clamp-v2");
    digest.update(source.source.encoded_sha256());
    digest.update(source.fingerprint);
    digest.update((frame as u64).to_le_bytes());
    for value in [rect.x, rect.y, rect.width, rect.height] {
        digest.update(value.to_le_bytes());
    }
    TextureKey {
        fingerprint: digest.finalize().into(),
        repeat: false,
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
        regions.push(FrameRegion {
            index: source.index,
            frame,
            rect,
            key: region_key(&source, frame, layout.rect()),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn authored_bytes(color: u16, duration: f32) -> Vec<u8> {
        // Same 8x4/two-block TEXS3 framing as the retained history fixture.
        let mut bytes = b"TEXV0005\0TEXI0001\0".to_vec();
        for value in [4_u32, 7, 8, 4, 8, 4, 0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(b"TEXB0004\0");
        for value in [1_u32, u32::MAX, 0, 1, 8, 4, 0, 0, 32] {
            bytes.extend(value.to_le_bytes());
        }
        for color in [color, 0x07e0_u16] {
            let mut block = [0_u8; 16];
            block[0] = 255;
            block[1] = 255;
            block[8..10].copy_from_slice(&color.to_le_bytes());
            bytes.extend(block);
        }
        bytes.extend(b"TEXS0003\0");
        for value in [2_u32, 2, 2] {
            bytes.extend(value.to_le_bytes());
        }
        for x in [0_f32, 4.] {
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(duration.to_le_bytes());
            for value in [x, 0., 2., 0., 0., 2.] {
                bytes.extend(value.to_le_bytes());
            }
        }
        bytes
    }

    fn source(bytes: &[u8], fingerprint: [u8; 32]) -> Result<Source> {
        let budget = lwe_scene::AtlasMemoryBudget::new(8 * 1024 * 1024)?;
        let mut reservation = budget.reserve_bytes(bytes.len())?;
        let bytes = bytes.to_vec();
        reservation.resize(bytes.capacity())?;
        Ok(Source {
            index: 0,
            fingerprint,
            source: Arc::new(TimedAtlasSource::from_budgeted_bytes(
                bytes,
                reservation,
                lwe_scene::ResourceOrigin::Project,
                &mut lwe_scene::TextureInspectionBudget::default(),
                &budget,
            )?),
        })
    }

    #[test]
    fn stale_prepared_fingerprint_cannot_alias_changed_bc3_pixels() -> Result<()> {
        let red = source(&authored_bytes(0xf800, 0.1), [7; 32])?;
        let blue = source(&authored_bytes(0x001f, 0.1), [7; 32])?;
        let rect = red.source.frame_neighborhood(0, 1)?.rect();
        assert_ne!(red.source.encoded_sha256(), blue.source.encoded_sha256());
        assert_ne!(region_key(&red, 0, rect), region_key(&blue, 0, rect));
        assert_ne!(
            red.source.decode_region(rect)?.rgba(),
            blue.source.decode_region(rect)?.rgba(),
        );
        Ok(())
    }

    #[test]
    fn prepared_namespace_still_separates_equal_source_pixels() -> Result<()> {
        let original = source(&authored_bytes(0xf800, 0.1), [7; 32])?;
        let mut other_namespace = original.clone();
        other_namespace.fingerprint = [8; 32];
        let rect = original.source.frame_neighborhood(0, 1)?.rect();
        assert_eq!(
            original.source.encoded_sha256(),
            other_namespace.source.encoded_sha256()
        );
        assert_ne!(
            region_key(&original, 0, rect),
            region_key(&other_namespace, 0, rect)
        );
        Ok(())
    }

    #[test]
    fn metadata_only_change_isolated_even_when_decoded_pixels_match() -> Result<()> {
        let fast = source(&authored_bytes(0xf800, 0.1), [7; 32])?;
        let slow = source(&authored_bytes(0xf800, 0.2), [7; 32])?;
        let rect = fast.source.frame_neighborhood(0, 1)?.rect();
        assert_eq!(
            fast.source.decode_region(rect)?.rgba(),
            slow.source.decode_region(rect)?.rgba()
        );
        assert_ne!(fast.source.encoded_sha256(), slow.source.encoded_sha256());
        assert_ne!(region_key(&fast, 0, rect), region_key(&slow, 0, rect));
        Ok(())
    }

    #[test]
    fn equal_content_in_distinct_physical_owners_preserves_region_key() -> Result<()> {
        let bytes = authored_bytes(0xf800, 0.1);
        let first = source(&bytes, [7; 32])?;
        let mut second = source(&bytes, [7; 32])?;
        second.index = 9;
        assert_ne!(first.source.allocation_id(), second.source.allocation_id());
        let rect = first.source.frame_neighborhood(0, 1)?.rect();
        assert_eq!(
            first.source.decode_region(rect)?.rgba(),
            second.source.decode_region(rect)?.rgba()
        );
        assert_eq!(region_key(&first, 0, rect), region_key(&second, 0, rect));
        Ok(())
    }

    #[test]
    fn frame_index_and_real_neighbor_roi_are_part_of_the_sampling_key() -> Result<()> {
        let source = source(&authored_bytes(0xf800, 0.1), [7; 32])?;
        let frame = source.source.frame_layout(0)?.rect();
        let neighbors = source.source.frame_neighborhood(0, 1)?.rect();
        assert_ne!(frame, neighbors);
        assert_ne!(
            region_key(&source, 0, frame),
            region_key(&source, 0, neighbors)
        );
        assert_ne!(
            region_key(&source, 0, neighbors),
            region_key(&source, 1, neighbors)
        );
        let second = source.source.frame_neighborhood(1, 1)?.rect();
        assert_ne!(
            region_key(&source, 0, neighbors),
            region_key(&source, 1, second)
        );
        Ok(())
    }

    fn wait_for_frame(
        worker: &mut TimedWorker,
        source: &Source,
        seq: u64,
        time: f64,
    ) -> Result<ReadyFrame> {
        let started = std::time::Instant::now();
        let mut requested = false;
        loop {
            if !requested {
                requested = worker.request(seq, time, std::slice::from_ref(source), Vec::new())?;
            }
            if requested && let Some(ready) = worker.take()? {
                return Ok(ready);
            }
            ensure!(
                started.elapsed() < std::time::Duration::from_secs(1),
                "Timed CPU fixture did not complete"
            );
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[test]
    fn same_selected_frame_across_worker_epochs_reuses_the_content_key() -> Result<()> {
        let source = source(&authored_bytes(0xf800, 0.1), [7; 32])?;
        let mut worker = TimedWorker::new()?;
        let first = wait_for_frame(&mut worker, &source, 1, 0.02)?;
        assert_eq!((first.seq, first.time), (1, 0.02));
        let key = first.regions[0].key;
        let frame = first.regions[0].frame;
        drop(first);
        let second = wait_for_frame(&mut worker, &source, 2, 0.07)?;
        assert_eq!((second.seq, second.time), (2, 0.07));
        assert_eq!(second.regions[0].frame, frame);
        assert_eq!(second.regions[0].key, key);
        Ok(())
    }
}
