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
use tracing::{debug, info};

use crate::config::EffectiveConfig;
use crate::core::types::{HwdecMode, VideoSource};

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
    /// Number of frame drops
    pub frames_dropped: u64,
    /// Current decode FPS
    pub decode_fps: f64,
}

/// Handle to a shared decoder
/// 
/// When all handles are dropped, the decoder is automatically stopped
#[derive(Clone)]
pub struct DecoderHandle {
    key: SourceKey,
    manager: Arc<RwLock<SharedDecodeManager>>,
}

impl DecoderHandle {
    /// Get current frame dimensions (width, height)
    pub fn dimensions(&self) -> Option<(i32, i32)> {
        let manager = self.manager.read().ok()?;
        manager.decoders.get(&self.key)?.dimensions()
    }
    
    /// Get decoder statistics
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

/// Shared frame buffer for decoded video frames
struct FrameBuffer {
    // TODO: Implement actual frame storage using Arc<[u8]> slices
    // For now, this is a placeholder
    _placeholder: (),
}

/// A shared decoder instance with reference counting
struct SharedDecoder {
    /// Reference count - number of consumers
    ref_count: usize,
    
    /// The actual MPV player instance
    /// TODO: Replace with actual MpvPlayer when integrated
    _player: (),
    
    /// Shared frame buffer
    _frame_buffer: Arc<Mutex<FrameBuffer>>,
    
    /// Decoder statistics
    stats: DecoderStats,
    
    /// Cached video dimensions
    dimensions: Option<(i32, i32)>,
}

impl SharedDecoder {
    fn new() -> Self {
        Self {
            ref_count: 0,
            _player: (),
            _frame_buffer: Arc::new(Mutex::new(FrameBuffer { _placeholder: () })),
            stats: DecoderStats::default(),
            dimensions: None,
        }
    }
    
    fn dimensions(&self) -> Option<(i32, i32)> {
        self.dimensions
    }
    
    fn stats(&self) -> DecoderStats {
        self.stats.clone()
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
}

impl SharedDecodeManager {
    /// Create a new manager
    fn new() -> Self {
        Self {
            decoders: HashMap::new(),
            active_decoders: 0,
            total_consumers: 0,
        }
    }
    
    /// Get the global manager instance
    pub fn global() -> Arc<RwLock<Self>> {
        static INSTANCE: std::sync::OnceLock<Arc<RwLock<SharedDecodeManager>>> = 
            std::sync::OnceLock::new();
        
        INSTANCE.get_or_init(|| {
            info!("🎬 Initializing Shared Decode Manager");
            Arc::new(RwLock::new(Self::new()))
        }).clone()
    }
    
    /// Acquire a decoder for the given configuration
    /// 
    /// If a decoder already exists for this source+params, returns a handle
    /// to the existing decoder. Otherwise, creates a new decoder.
    pub fn acquire_decoder(
        manager: Arc<RwLock<Self>>,
        config: &EffectiveConfig,
    ) -> Result<DecoderHandle> {
        let key = SourceKey::from_config(config);
        
        // First check if decoder exists (read lock)
        {
            let mgr = manager.read().unwrap();
            if mgr.decoders.contains_key(&key) {
                debug!("♻️  Reusing existing decoder for {:?}", key.source);
            }
        }
        
        // Acquire or create decoder (write lock)
        {
            let mut mgr = manager.write().unwrap();
            
            let is_new = !mgr.decoders.contains_key(&key);
            if is_new {
                info!("🆕 Creating new shared decoder for {:?}", key.source);
                mgr.active_decoders += 1;
                mgr.decoders.insert(key.clone(), SharedDecoder::new());
            }
            
            let decoder = mgr.decoders.get_mut(&key).unwrap();
            decoder.ref_count += 1;
            decoder.stats.consumer_count = decoder.ref_count;
            mgr.total_consumers += 1;
            
            info!(
                "📊 Decoder stats: {} decoders, {} total consumers",
                mgr.active_decoders, mgr.total_consumers
            );
        }
        
        Ok(DecoderHandle {
            key,
            manager: manager.clone(),
        })
    }
    
    /// Release a decoder (called when handle is dropped)
    fn release_decoder(&mut self, key: &SourceKey) {
        if let Some(decoder) = self.decoders.get_mut(key) {
            decoder.ref_count -= 1;
            decoder.stats.consumer_count = decoder.ref_count;
            self.total_consumers -= 1;
            
            debug!("📉 Decoder ref count: {} for {:?}", decoder.ref_count, key.source);
            
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
    pub fn global_stats(&self) -> GlobalStats {
        GlobalStats {
            active_decoders: self.active_decoders,
            total_consumers: self.total_consumers,
            decoders: self.decoders.iter()
                .map(|(key, decoder)| (key.clone(), decoder.stats()))
                .collect(),
        }
    }
}

/// Global statistics across all decoders
#[derive(Debug, Clone)]
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
    use crate::core::types::{LayoutMode, VideoSource};
    use crate::config::PowerConfig;
    
    fn make_test_config(path: &str) -> EffectiveConfig {
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
    
    #[test]
    fn test_decoder_reference_counting() {
        let manager = SharedDecodeManager::global();
        let config = make_test_config("/test/video.mp4");
        
        // Acquire first handle
        let handle1 = SharedDecodeManager::acquire_decoder(manager.clone(), &config).unwrap();
        {
            let mgr = manager.read().unwrap();
            assert_eq!(mgr.active_decoders, 1);
            assert_eq!(mgr.total_consumers, 1);
        }
        
        // Acquire second handle (should reuse decoder)
        let handle2 = SharedDecodeManager::acquire_decoder(manager.clone(), &config).unwrap();
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
    fn test_multiple_different_sources() {
        let manager = SharedDecodeManager::global();
        let config1 = make_test_config("/test/video1.mp4");
        let config2 = make_test_config("/test/video2.mp4");
        
        let _handle1 = SharedDecodeManager::acquire_decoder(manager.clone(), &config1).unwrap();
        let _handle2 = SharedDecodeManager::acquire_decoder(manager.clone(), &config2).unwrap();
        
        let mgr = manager.read().unwrap();
        assert_eq!(mgr.active_decoders, 2);
        assert_eq!(mgr.total_consumers, 2);
    }
}
