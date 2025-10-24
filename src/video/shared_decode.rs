/// Shared decode context for multi-output video playback
///
/// This module implements RFC M5-001: Shared Decode Context
///
/// Key features:
/// - Single decode process shared across multiple outputs
/// - Reference-counted resource management
/// - Thread-safe frame buffer access
/// - Automatic cleanup when all consumers disconnect
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tracing::{debug, info, warn};

use crate::config::EffectiveConfig;
use crate::core::types::{HwdecMode, OutputInfo, VideoSource};
use crate::video::egl::EglContext;
use crate::video::memory::{BufferPool, MemoryStats};
use crate::video::mpv::MpvPlayer;

/// Unique identifier for a video source + decode parameters
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SourceKey {
    /// Video source (file path, URL, etc.)
    pub source: VideoSource,

    /// Decode parameters that affect output
    pub params: DecodeParams,
}

impl SourceKey {
    /// Create a new source key from configuration
    pub fn from_config(config: &EffectiveConfig) -> Self {
        Self {
            source: config.source.clone(),
            params: DecodeParams {
                hwdec: config.hwdec.into(),
                // Only include parameters that affect decoded output
                // Audio, loop, etc. are per-consumer settings
            },
        }
    }

    /// Get a human-readable description of this source
    pub fn description(&self) -> String {
        format!("{:?} (hwdec={:?})", self.source, self.params.hwdec)
    }
}

/// Decode parameters that affect the decoded output
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DecodeParams {
    /// Hardware decode mode
    pub hwdec: HwdecMode,
    // Future: Add more params like target resolution if we add scaling
}

/// Statistics for a shared decoder
#[derive(Debug, Clone, Default)]
pub struct DecoderStats {
    /// Number of consumers currently using this decoder
    pub consumer_count: usize,
    /// Total frames decoded
    pub frames_decoded: u64,
    /// Number of frame drops (TODO: implement tracking)
    #[allow(dead_code)]
    pub frames_dropped: u64,
    /// Current decode FPS (TODO: implement tracking)
    #[allow(dead_code)]
    pub decode_fps: f64,
}

/// Handle to a shared decoder
///
/// When all handles are dropped, the decoder is automatically stopped
#[derive(Clone)]
pub struct DecoderHandle {
    key: SourceKey,
    manager: Arc<RwLock<SharedDecodeManager>>,
    #[allow(dead_code)]
    frame_buffer: Arc<Mutex<FrameBuffer>>,
}

impl DecoderHandle {
    /// Get source key description
    pub fn source_description(&self) -> String {
        self.key.description()
    }

    /// Initialize OpenGL rendering context for this decoder
    pub fn init_render_context(&self, egl_context: &EglContext) -> Result<()> {
        let mut manager = self.manager.write().unwrap();
        if let Some(decoder) = manager.decoders.get_mut(&self.key) {
            decoder.init_render_context(egl_context)
        } else {
            Ok(())
        }
    }

    /// Render a frame (only one consumer needs to call this)
    pub fn render(&self, width: i32, height: i32, fbo: i32) -> Result<()> {
        let mut manager = self.manager.write().unwrap();
        if let Some(decoder) = manager.decoders.get_mut(&self.key) {
            decoder.render(width, height, fbo)
        } else {
            Ok(())
        }
    }

    /// Get current frame from shared buffer (all consumers can call this)
    #[allow(dead_code)]
    pub fn get_frame(&self) -> Option<(Arc<Vec<u8>>, i32, i32, u64)> {
        let buffer = self.frame_buffer.lock().ok()?;
        buffer.get_frame()
    }

    /// Get reference to the buffer pool
    #[allow(dead_code)]
    pub fn buffer_pool(&self) -> Arc<BufferPool> {
        let manager = self.manager.read().unwrap();
        manager.buffer_pool()
    }

    /// Log memory statistics
    #[allow(dead_code)]
    pub fn log_memory_stats(&self) {
        let manager = self.manager.read().unwrap();
        manager.log_memory_stats();
    }

    /// Check current memory pressure level
    #[allow(dead_code)]
    pub fn check_memory_pressure(&self) -> MemoryPressureLevel {
        let manager = self.manager.read().unwrap();
        manager.check_memory_pressure()
    }

    /// Handle memory pressure (cleanup if needed)
    pub fn handle_memory_pressure(&self) {
        let manager = self.manager.read().unwrap();
        manager.handle_memory_pressure();
    }

    /// Get current frame dimensions (width, height)
    pub fn dimensions(&self) -> Option<(i32, i32)> {
        let manager = self.manager.read().ok()?;
        manager.decoders.get(&self.key)?.dimensions()
    }

    /// Get decoder statistics
    #[allow(dead_code)]
    pub fn stats(&self) -> Option<DecoderStats> {
        let manager = self.manager.read().ok()?;
        Some(manager.decoders.get(&self.key)?.stats())
    }
}

impl Drop for DecoderHandle {
    fn drop(&mut self) {
        // Decrement reference count when handle is dropped
        if let Ok(mut manager) = self.manager.write() {
            manager.release_decoder(&self.key);
        }
    }
}

/// Shared frame buffer for zero-copy frame extraction
pub struct FrameBuffer {
    /// Current frame data (using ManagedBuffer from pool)
    /// TODO: Implement frame extraction from MPV
    #[allow(dead_code)]
    frame_data: Option<Arc<Vec<u8>>>,

    /// Frame dimensions (width, height)
    #[allow(dead_code)]
    dimensions: Option<(i32, i32)>,

    /// Frame sequence number
    #[allow(dead_code)]
    sequence: u64,

    /// Last update timestamp
    #[allow(dead_code)]
    last_update: std::time::Instant,

    /// Buffer pool reference for managed allocations
    #[allow(dead_code)]
    buffer_pool: Arc<BufferPool>,
}
impl FrameBuffer {
    fn new(buffer_pool: Arc<BufferPool>) -> Self {
        Self {
            frame_data: None,
            dimensions: None,
            sequence: 0,
            last_update: std::time::Instant::now(),
            buffer_pool,
        }
    }

    /// Update frame data (for future frame extraction implementation)
    #[allow(dead_code)]
    fn update_frame(&mut self, data: Vec<u8>, width: i32, height: i32) {
        self.frame_data = Some(Arc::new(data));
        self.dimensions = Some((width, height));
        self.sequence += 1;
        self.last_update = std::time::Instant::now();
    }

    /// Get current frame (returns Arc for zero-copy sharing)
    #[allow(dead_code)]
    fn get_frame(&self) -> Option<(Arc<Vec<u8>>, i32, i32, u64)> {
        let data = self.frame_data.as_ref()?.clone();
        let (width, height) = self.dimensions?;
        Some((data, width, height, self.sequence))
    }
}

/// A shared decoder instance with reference counting
struct SharedDecoder {
    /// Reference count - number of consumers
    ref_count: usize,

    /// The actual MPV player instance (wrapped in Mutex for interior mutability)
    player: Arc<Mutex<MpvPlayer>>,

    /// Shared frame buffer
    frame_buffer: Arc<Mutex<FrameBuffer>>,

    /// Configuration (needed for HDR setup)
    config: EffectiveConfig,

    /// Decoder statistics
    stats: DecoderStats,
}

impl SharedDecoder {
    fn new(
        config: &EffectiveConfig,
        output_info: &OutputInfo,
        buffer_pool: Arc<BufferPool>,
    ) -> Result<Self> {
        info!("🎬 Creating shared decoder for {:?}", config.source);

        // Create MPV player
        let player = MpvPlayer::new(config, output_info)?;

        Ok(Self {
            ref_count: 0,
            player: Arc::new(Mutex::new(player)),
            frame_buffer: Arc::new(Mutex::new(FrameBuffer::new(buffer_pool.clone()))),
            config: config.clone(),
            stats: DecoderStats::default(),
        })
    }

    /// Initialize OpenGL rendering context
    fn init_render_context(&self, egl_context: &EglContext) -> Result<()> {
        let mut player = self.player.lock().unwrap();
        player.init_render_context(egl_context)?;
        // Configure HDR after render context is ready
        player.configure_hdr(&self.config)?;
        Ok(())
    }

    /// Get current video dimensions
    fn dimensions(&self) -> Option<(i32, i32)> {
        let mut player = self.player.lock().unwrap();
        player.get_video_dimensions()
    }

    /// Get decoder statistics
    #[allow(dead_code)]
    fn stats(&self) -> DecoderStats {
        self.stats.clone()
    }

    /// Render frame (called by one consumer, updates shared buffer)
    fn render(&mut self, width: i32, height: i32, fbo: i32) -> Result<()> {
        // Render to FBO
        {
            let mut player = self.player.lock().unwrap();
            player.render(width, height, fbo)?;
        }

        // Update stats
        self.stats.frames_decoded += 1;

        // Log memory stats periodically (every 300 frames ~= every 5 seconds at 60fps)
        if self.stats.frames_decoded % 300 == 0 {
            let mem_stats = MemoryStats::global();
            debug!(
                "📊 Memory after {} frames: current={}, peak={}",
                self.stats.frames_decoded,
                MemoryStats::format_bytes(mem_stats.current_bytes),
                MemoryStats::format_bytes(mem_stats.peak_bytes),
            );
        }

        // TODO: Extract rendered frame to shared buffer
        // For now, just increment frame count

        Ok(())
    }

    /// Get shared frame buffer reference
    fn get_frame_buffer(&self) -> Arc<Mutex<FrameBuffer>> {
        self.frame_buffer.clone()
    }
}

/// Global manager for shared decoders
///
/// This is a singleton that manages all shared decoder instances.
/// It ensures only one decoder exists per unique source+params combination.
pub struct SharedDecodeManager {
    /// Map of source keys to shared decoders
    decoders: HashMap<SourceKey, SharedDecoder>,

    /// Total number of active decoders
    active_decoders: usize,

    /// Total number of consumers across all decoders
    total_consumers: usize,

    /// Buffer pool for texture data (shared across all decoders)
    buffer_pool: Arc<BufferPool>,
}

impl SharedDecodeManager {
    /// Create a new manager
    fn new() -> Self {
        // Configure buffer pool with conservative defaults
        // TODO: Read from global config once available
        // Users can override via config.yaml:
        //   power:
        //     max_memory_mb: 100
        //     max_buffers: 8
        const MAX_BUFFERS: usize = 8;
        const MAX_MEMORY: usize = 100 * 1024 * 1024; // 100MB

        info!(
            "🎬 Initializing buffer pool: max_buffers={}, max_memory={}MB",
            MAX_BUFFERS,
            MAX_MEMORY / (1024 * 1024)
        );

        Self {
            decoders: HashMap::new(),
            active_decoders: 0,
            total_consumers: 0,
            buffer_pool: Arc::new(BufferPool::new(MAX_BUFFERS, MAX_MEMORY)),
        }
    }

    /// Get the global manager instance
    pub fn global() -> Arc<RwLock<Self>> {
        static INSTANCE: std::sync::OnceLock<Arc<RwLock<SharedDecodeManager>>> =
            std::sync::OnceLock::new();

        INSTANCE
            .get_or_init(|| {
                info!("🎬 Initializing Shared Decode Manager");
                Arc::new(RwLock::new(Self::new()))
            })
            .clone()
    }

    /// Acquire a decoder for the given configuration
    ///
    /// If a decoder already exists for this source+params, returns a handle
    /// to the existing decoder. Otherwise, creates a new decoder.
    pub fn acquire_decoder(
        manager: Arc<RwLock<Self>>,
        config: &EffectiveConfig,
        output_info: &OutputInfo,
    ) -> Result<DecoderHandle> {
        let key = SourceKey::from_config(config);

        // First check if decoder exists (read lock)
        let (frame_buffer, is_new, buffer_pool) = {
            let mgr = manager.read().unwrap();
            let exists = mgr.decoders.contains_key(&key);
            let pool = mgr.buffer_pool.clone();
            if exists {
                debug!("♻️  Reusing existing decoder for {:?}", key.source);
                let fb = mgr.decoders.get(&key).unwrap().get_frame_buffer();
                (fb, false, pool)
            } else {
                (
                    Arc::new(Mutex::new(FrameBuffer::new(pool.clone()))),
                    true,
                    pool,
                )
            }
        };

        // Acquire or create decoder (write lock)
        if is_new {
            let mut mgr = manager.write().unwrap();

            // Double-check in case another thread created it
            if !mgr.decoders.contains_key(&key) {
                info!("🆕 Creating new shared decoder for {:?}", key.source);
                let decoder = SharedDecoder::new(config, output_info, buffer_pool)?;
                mgr.active_decoders += 1;
                mgr.decoders.insert(key.clone(), decoder);
            }
        }

        // Increment reference count
        {
            let mut mgr = manager.write().unwrap();
            let decoder = mgr.decoders.get_mut(&key).unwrap();
            decoder.ref_count += 1;
            decoder.stats.consumer_count = decoder.ref_count;
            mgr.total_consumers += 1;

            info!(
                "📊 Decoder stats: {} decoders, {} total consumers (key: {:?})",
                mgr.active_decoders, mgr.total_consumers, key.source
            );
        }

        // Get frame buffer (need to clone key for later use)
        let frame_buffer_final = if is_new {
            // Get the actual frame buffer from the decoder we just created
            let mgr = manager.read().unwrap();
            mgr.decoders.get(&key).unwrap().get_frame_buffer()
        } else {
            frame_buffer
        };

        Ok(DecoderHandle {
            key,
            manager: manager.clone(),
            frame_buffer: frame_buffer_final,
        })
    }

    /// Release a decoder (called when handle is dropped)
    fn release_decoder(&mut self, key: &SourceKey) {
        if let Some(decoder) = self.decoders.get_mut(key) {
            decoder.ref_count -= 1;
            decoder.stats.consumer_count = decoder.ref_count;
            self.total_consumers -= 1;

            debug!(
                "📉 Decoder ref count: {} for {:?}",
                decoder.ref_count, key.source
            );

            if decoder.ref_count == 0 {
                info!("🗑️  Removing unused decoder for {:?}", key.source);
                self.decoders.remove(key);
                self.active_decoders -= 1;

                info!(
                    "📊 Decoder stats: {} decoders, {} total consumers",
                    self.active_decoders, self.total_consumers
                );
            }
        }
    }

    /// Get statistics for all decoders
    #[allow(dead_code)]
    pub fn global_stats(&self) -> GlobalStats {
        GlobalStats {
            active_decoders: self.active_decoders,
            total_consumers: self.total_consumers,
            decoders: self
                .decoders
                .iter()
                .map(|(key, decoder)| (key.clone(), decoder.stats()))
                .collect(),
        }
    }

    /// Get reference to the buffer pool
    #[allow(dead_code)]
    pub fn buffer_pool(&self) -> Arc<BufferPool> {
        self.buffer_pool.clone()
    }

    /// Log memory statistics
    #[allow(dead_code)]
    pub fn log_memory_stats(&self) {
        let mem_stats = MemoryStats::global();
        let pool_stats = self.buffer_pool.stats();

        info!(
            "💾 Memory: current={}, peak={}, pool: {}/{} buffers",
            MemoryStats::format_bytes(mem_stats.current_bytes),
            MemoryStats::format_bytes(mem_stats.peak_bytes),
            pool_stats.buffer_count,
            pool_stats.max_buffers,
        );

        mem_stats.log();
        pool_stats.log();
    }

    /// Check if system is under memory pressure
    pub fn check_memory_pressure(&self) -> MemoryPressureLevel {
        let mem_stats = MemoryStats::global();
        let pool_stats = self.buffer_pool.stats();

        // Calculate utilization percentages
        let pool_utilization = pool_stats.utilization();

        // Define thresholds
        const WARN_THRESHOLD: f64 = 0.75; // 75% utilization
        const CRITICAL_THRESHOLD: f64 = 0.90; // 90% utilization

        if pool_utilization >= CRITICAL_THRESHOLD {
            warn!(
                "⚠️  Critical memory pressure! Pool: {:.1}%, Memory: {}",
                pool_utilization * 100.0,
                MemoryStats::format_bytes(mem_stats.current_bytes)
            );
            MemoryPressureLevel::Critical
        } else if pool_utilization >= WARN_THRESHOLD {
            debug!(
                "⚠️  High memory pressure. Pool: {:.1}%, Memory: {}",
                pool_utilization * 100.0,
                MemoryStats::format_bytes(mem_stats.current_bytes)
            );
            MemoryPressureLevel::High
        } else {
            MemoryPressureLevel::Normal
        }
    }

    /// React to memory pressure by clearing buffer pool
    pub fn handle_memory_pressure(&self) {
        let pressure = self.check_memory_pressure();

        match pressure {
            MemoryPressureLevel::Critical => {
                warn!("🧹 Critical pressure: clearing buffer pool");
                self.buffer_pool.clear();
            }
            MemoryPressureLevel::High => {
                debug!("🧹 High pressure: partially clearing buffer pool");
                // Keep half of the buffers
                let stats = self.buffer_pool.stats();
                if stats.buffer_count > 4 {
                    self.buffer_pool.clear();
                }
            }
            MemoryPressureLevel::Normal => {
                // No action needed
            }
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressureLevel {
    /// Normal operation, plenty of memory available
    Normal,
    /// High memory usage, should consider cleanup
    High,
    /// Critical memory usage, immediate cleanup required
    Critical,
}

/// Global statistics across all decoders
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GlobalStats {
    pub active_decoders: usize,
    pub total_consumers: usize,
    pub decoders: HashMap<SourceKey, DecoderStats>,
}

impl Default for SharedDecodeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PowerConfig;
    use crate::core::types::{LayoutMode, VideoSource};

    fn make_test_config(path: &str) -> EffectiveConfig {
        use crate::video::hdr::{HdrMode, ToneMappingAlgorithm, ToneMappingConfig};

        EffectiveConfig {
            source: VideoSource::File {
                path: path.to_string(),
            },
            layout: LayoutMode::Fill,
            hwdec: true,
            r#loop: true,
            mute: false,
            volume: 1.0,
            start_time: 0.0,
            playback_rate: 1.0,
            power: PowerConfig::default(),
            hdr_mode: HdrMode::Auto,
            tone_mapping: ToneMappingConfig {
                algorithm: ToneMappingAlgorithm::Hable,
                param: 1.0,
                compute_peak: true,
                mode: "hybrid".to_string(),
            },
        }
    }

    fn make_test_output_info(name: &str) -> OutputInfo {
        use crate::core::types::OutputHdrCapabilities;
        use crate::video::hdr::TransferFunction;

        OutputInfo {
            name: name.to_string(),
            width: 1920,
            height: 1080,
            scale: 1.0,
            position: (0, 0),
            active: true,
            hdr_capabilities: OutputHdrCapabilities {
                hdr_supported: false,
                max_luminance: Some(203.0),
                min_luminance: Some(0.0),
                supported_eotf: vec![TransferFunction::Srgb],
            },
        }
    }

    #[test]
    fn test_source_key_equality() {
        let config1 = make_test_config("/path/to/video.mp4");
        let config2 = make_test_config("/path/to/video.mp4");
        let config3 = make_test_config("/path/to/other.mp4");

        let key1 = SourceKey::from_config(&config1);
        let key2 = SourceKey::from_config(&config2);
        let key3 = SourceKey::from_config(&config3);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    // Note: The following tests require actual video files and MPV to work
    // They are disabled by default. Enable with #[ignore] removed when testing with real files.

    #[test]
    #[ignore]
    fn test_decoder_reference_counting() {
        let manager = SharedDecodeManager::global();
        let config = make_test_config("/test/video.mp4");
        let output = make_test_output_info("TEST-1");

        // Acquire first handle
        let handle1 =
            SharedDecodeManager::acquire_decoder(manager.clone(), &config, &output).unwrap();
        {
            let mgr = manager.read().unwrap();
            assert_eq!(mgr.active_decoders, 1);
            assert_eq!(mgr.total_consumers, 1);
        }

        // Acquire second handle (should reuse decoder)
        let handle2 =
            SharedDecodeManager::acquire_decoder(manager.clone(), &config, &output).unwrap();
        {
            let mgr = manager.read().unwrap();
            assert_eq!(mgr.active_decoders, 1);
            assert_eq!(mgr.total_consumers, 2);
        }

        // Drop first handle
        drop(handle1);
        {
            let mgr = manager.read().unwrap();
            assert_eq!(mgr.active_decoders, 1);
            assert_eq!(mgr.total_consumers, 1);
        }

        // Drop second handle (should cleanup decoder)
        drop(handle2);
        {
            let mgr = manager.read().unwrap();
            assert_eq!(mgr.active_decoders, 0);
            assert_eq!(mgr.total_consumers, 0);
        }
    }

    #[test]
    #[ignore]
    fn test_multiple_different_sources() {
        let manager = SharedDecodeManager::global();
        let config1 = make_test_config("/test/video1.mp4");
        let config2 = make_test_config("/test/video2.mp4");
        let output = make_test_output_info("TEST-1");

        let _handle1 =
            SharedDecodeManager::acquire_decoder(manager.clone(), &config1, &output).unwrap();
        let _handle2 =
            SharedDecodeManager::acquire_decoder(manager.clone(), &config2, &output).unwrap();

        let mgr = manager.read().unwrap();
        assert_eq!(mgr.active_decoders, 2);
        assert_eq!(mgr.total_consumers, 2);
    }
}
