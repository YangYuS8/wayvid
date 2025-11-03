# RFC: Shared Decode Context for Multi-Output

**RFC Number**: M5-001  
**Author**: YangYuS8  
**Status**: 📝 Draft  
**Created**: 2025-10-23  
**Updated**: 2025-10-23

---

## Summary

Implement a shared decode context that allows multiple outputs to consume the same decoded video frames, significantly reducing CPU usage when the same video is displayed on multiple monitors.

## Motivation

**Problem**: Currently, when displaying the same video on multiple outputs, each output creates its own MPV player instance. This means:

- The same video is decoded N times (once per output)
- CPU usage scales linearly with output count
- Memory usage is unnecessarily high
- Battery drain on laptops is significant

**Example Scenario**:
- 3 monitors showing the same 4K wallpaper
- CPU usage: ~30% (3x ~10% per decode)
- With shared decode: ~12% (1x decode + 3x render overhead)
- **Savings: 60% CPU reduction**

## Design Overview

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    SharedDecodeManager                      │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Video Source Hash → MpvPlayer + FrameBuffer           │ │
│  │  "~/vid.mp4:1920x1080" → Player1 + Buffer1            │ │
│  │  "~/other.mp4:..." → Player2 + Buffer2                 │ │
│  └────────────────────────────────────────────────────────┘ │
│                           │                                  │
│                           ├──────────┬──────────┬──────────┐ │
│                           ▼          ▼          ▼          ▼ │
│                      Consumer1  Consumer2  Consumer3  Consumer4│
│                      (Output1)  (Output2)  (Output3)  (Output4)│
└─────────────────────────────────────────────────────────────┘
```

### Key Components

1. **SharedDecodeManager**: Global singleton managing all decoders
2. **MpvPlayer** (shared): One player instance per unique video source
3. **FrameBuffer**: Shared frame storage with reference counting
4. **Consumer**: Per-output consumer of shared frames
5. **SourceKey**: Hash of video source + decode parameters

### Data Flow

```
1. Output requests video: "~/wallpaper.mp4"
2. Manager checks if decoder exists for this source
3. If exists:
   - Increment reference count
   - Register new consumer
   - Return handle to shared frame buffer
4. If not exists:
   - Create new MpvPlayer
   - Start decoding
   - Create FrameBuffer
   - Register consumer
5. On frame ready:
   - Decoder writes to FrameBuffer
   - Notify all consumers
6. Consumer renders:
   - Lock FrameBuffer
   - Copy/upload to output's texture
   - Unlock FrameBuffer
7. Output destroyed:
   - Decrement reference count
   - If count == 0: Stop decoder, free resources
```

## Detailed Design

### 1. Source Identification

```rust
#[derive(Hash, Eq, PartialEq, Clone)]
struct SourceKey {
    /// Video source (file path, URL, etc.)
    source: VideoSource,
    
    /// Decode parameters that affect output
    params: DecodeParams,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct DecodeParams {
    /// Hardware decode mode
    hwdec: bool,
    
    /// Start time (for trimmed videos)
    start_time: f64,
    
    /// Video track selection
    vid: Option<i64>,
    
    // NOTE: layout, volume, etc. are per-output, not decode params
}
```

**Key Decision**: Two outputs showing the same video with different layouts/volumes share the same decoder, but different start times create separate decoders.

### 2. Shared Decode Manager

```rust
pub struct SharedDecodeManager {
    /// Active decoders: SourceKey → Decoder
    decoders: Arc<Mutex<HashMap<SourceKey, Arc<SharedDecoder>>>>,
    
    /// Consumer registry: OutputId → Consumer
    consumers: Arc<Mutex<HashMap<u32, Consumer>>>,
    
    /// Global enable flag
    enabled: bool,
}

impl SharedDecodeManager {
    pub fn get_or_create_decoder(
        &self, 
        source: VideoSource, 
        params: DecodeParams
    ) -> Result<Arc<SharedDecoder>> {
        let key = SourceKey { source, params };
        
        let mut decoders = self.decoders.lock().unwrap();
        
        if let Some(decoder) = decoders.get(&key) {
            decoder.add_ref();
            return Ok(Arc::clone(decoder));
        }
        
        // Create new decoder
        let decoder = Arc::new(SharedDecoder::new(key.clone())?);
        decoders.insert(key, Arc::clone(&decoder));
        
        Ok(decoder)
    }
    
    pub fn register_consumer(
        &self,
        output_id: u32,
        decoder: Arc<SharedDecoder>,
    ) -> Consumer {
        let consumer = Consumer::new(output_id, decoder);
        self.consumers.lock().unwrap().insert(output_id, consumer.clone());
        consumer
    }
    
    pub fn unregister_consumer(&self, output_id: u32) {
        if let Some(consumer) = self.consumers.lock().unwrap().remove(&output_id) {
            consumer.decoder.remove_ref();
            
            // Cleanup decoder if no more consumers
            if consumer.decoder.ref_count() == 0 {
                self.decoders.lock().unwrap().remove(&consumer.decoder.key);
            }
        }
    }
}
```

### 3. Shared Decoder

```rust
pub struct SharedDecoder {
    /// Source identification
    key: SourceKey,
    
    /// The actual MPV player
    player: Arc<Mutex<MpvPlayer>>,
    
    /// Shared frame buffer
    frame_buffer: Arc<Mutex<FrameBuffer>>,
    
    /// Reference count (number of consumers)
    ref_count: Arc<AtomicUsize>,
    
    /// Frame ready notifier
    frame_ready: Arc<Condvar>,
}

impl SharedDecoder {
    pub fn new(key: SourceKey) -> Result<Self> {
        let player = MpvPlayer::new()?;
        player.configure_for_decode(&key.params)?;
        player.load_source(&key.source)?;
        
        Ok(Self {
            key,
            player: Arc::new(Mutex::new(player)),
            frame_buffer: Arc::new(Mutex::new(FrameBuffer::new())),
            ref_count: Arc::new(AtomicUsize::new(1)),
            frame_ready: Arc::new(Condvar::new()),
        })
    }
    
    pub fn add_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn remove_ref(&self) {
        self.ref_count.fetch_sub(1, Ordering::SeqCst);
    }
    
    pub fn ref_count(&self) -> usize {
        self.ref_count.load(Ordering::SeqCst)
    }
    
    /// Called by MPV render thread when frame is ready
    pub fn on_frame_ready(&self, frame: VideoFrame) {
        let mut buffer = self.frame_buffer.lock().unwrap();
        buffer.update(frame);
        self.frame_ready.notify_all();
    }
}
```

### 4. Frame Buffer

```rust
pub struct FrameBuffer {
    /// Current frame data
    frame: Option<VideoFrame>,
    
    /// Frame sequence number
    seq: u64,
    
    /// Timestamp
    pts: f64,
}

pub struct VideoFrame {
    /// OpenGL texture ID (uploaded from MPV)
    texture: GLuint,
    
    /// Frame dimensions
    width: i32,
    height: i32,
    
    /// Pixel format
    format: PixelFormat,
}

impl FrameBuffer {
    pub fn update(&mut self, frame: VideoFrame) {
        self.frame = Some(frame);
        self.seq += 1;
        self.pts = frame.pts;
    }
    
    pub fn get_frame(&self) -> Option<&VideoFrame> {
        self.frame.as_ref()
    }
}
```

### 5. Consumer

```rust
pub struct Consumer {
    /// Output ID this consumer belongs to
    output_id: u32,
    
    /// Reference to shared decoder
    decoder: Arc<SharedDecoder>,
    
    /// Last rendered frame sequence
    last_seq: AtomicU64,
}

impl Consumer {
    pub fn wait_for_frame(&self, timeout: Duration) -> Option<VideoFrame> {
        let buffer = self.decoder.frame_buffer.lock().unwrap();
        
        // Wait for new frame
        let result = self.decoder.frame_ready
            .wait_timeout(buffer, timeout)
            .unwrap();
        
        let buffer = result.0;
        if let Some(frame) = buffer.get_frame() {
            let seq = buffer.seq;
            if seq > self.last_seq.load(Ordering::SeqCst) {
                self.last_seq.store(seq, Ordering::SeqCst);
                return Some(frame.clone());
            }
        }
        
        None
    }
    
    pub fn render_to_surface(&self, surface: &WaylandSurface) -> Result<()> {
        if let Some(frame) = self.wait_for_frame(Duration::from_millis(16)) {
            surface.render_texture(frame.texture, frame.width, frame.height)?;
        }
        Ok(())
    }
}
```

## Configuration

### YAML Config

```yaml
# Enable shared decode (default: true)
shared_decode: true

# Maximum decoders before forcing shared mode
max_decoders: 3

# Per-output still allows overrides
per_output:
  "HDMI-1":
    source:
      type: File
      path: "~/wallpaper.mp4"  # Will share with DP-1
  "DP-1":
    source:
      type: File
      path: "~/wallpaper.mp4"  # Shares decoder with HDMI-1
```

### Runtime Control

```bash
# Check shared decode status
wayvid-ctl decode-status
# Output: 2 decoders, 4 consumers
#   Decoder 1: ~/wallpaper.mp4 (3 consumers: HDMI-1, DP-1, DP-2)
#   Decoder 2: ~/other.mp4 (1 consumer: HDMI-2)

# Disable shared decode at runtime
wayvid-ctl set-shared-decode false

# Re-enable
wayvid-ctl set-shared-decode true
```

## Performance Implications

### CPU Usage

**Before (4 outputs, same video)**:
- 4x decode: ~40% CPU
- 4x render: ~4% CPU
- Total: ~44% CPU

**After (shared decode)**:
- 1x decode: ~10% CPU
- 4x render: ~4% CPU
- Overhead: ~2% CPU (synchronization)
- Total: ~16% CPU
- **Savings: 64%**

### Memory Usage

**Before**:
- 4x MPV contexts: ~400MB
- 4x frame buffers: ~100MB
- Total: ~500MB

**After**:
- 1x MPV context: ~100MB
- 1x shared frame buffer: ~25MB
- 4x consumer state: ~10MB
- Total: ~135MB
- **Savings: 73%**

### Latency

**Frame Delivery**:
- Decode: 16ms (same)
- Notify consumers: <1ms
- Consumer render: 2ms per output
- Total: ~16ms (no added latency)

**Synchronization Overhead**:
- Mutex contention: <0.1ms per frame
- Condvar wake: <0.05ms per consumer
- Negligible impact on 60fps playback

## Edge Cases & Handling

### 1. Different Decode Parameters

**Scenario**: Two outputs want the same video but with different start times.

**Behavior**: Create separate decoders (different SourceKey).

**Example**:
```yaml
per_output:
  "HDMI-1":
    source:
      type: File
      path: "~/vid.mp4"
    start_time: 0.0  # Decoder 1
  "DP-1":
    source:
      type: File
      path: "~/vid.mp4"
    start_time: 10.0  # Decoder 2 (different start)
```

### 2. Hotplug During Playback

**Scenario**: Output disconnected while decoder is running.

**Handling**:
1. Consumer detects output lost
2. Unregister consumer
3. Decrement decoder ref count
4. If ref count == 0: Stop decoder
5. Other consumers unaffected

### 3. Config Reload

**Scenario**: Config changes output sources during runtime.

**Handling**:
1. Unregister old consumer
2. Register new consumer (may reuse decoder)
3. Old decoder cleaned up if unused
4. Smooth transition without dropping frames

### 4. Decode Failure

**Scenario**: Shared decoder encounters error.

**Handling**:
1. Mark decoder as failed
2. Notify all consumers
3. Each consumer falls back to independent decode
4. Error logged once (not per output)

### 5. Mixed Shared/Independent

**Scenario**: Some outputs share, some don't (e.g., different videos).

**Behavior**: Works transparently. Manager tracks both shared and independent decoders.

## Migration Path

### Phase 1: Implement (Week 1)
- Add SharedDecodeManager skeleton
- Implement SourceKey and hashing
- Create SharedDecoder wrapper

### Phase 2: Integration (Week 1)
- Modify WaylandSurface to use Consumer
- Update config parsing for shared_decode flag
- Add enable/disable logic

### Phase 3: Testing (Week 1)
- Unit tests for Manager, Decoder, Consumer
- Integration tests with mock outputs
- Performance benchmarking

### Phase 4: Refinement (Week 2)
- Handle edge cases
- Optimize locking strategy
- Add telemetry

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_shared_decode_manager() {
    let manager = SharedDecodeManager::new();
    
    // Same source → same decoder
    let decoder1 = manager.get_or_create_decoder(source.clone(), params.clone()).unwrap();
    let decoder2 = manager.get_or_create_decoder(source.clone(), params.clone()).unwrap();
    assert_eq!(Arc::ptr_eq(&decoder1, &decoder2), true);
    assert_eq!(decoder1.ref_count(), 2);
}

#[test]
fn test_consumer_cleanup() {
    let manager = SharedDecodeManager::new();
    let decoder = manager.get_or_create_decoder(source, params).unwrap();
    
    let consumer1 = manager.register_consumer(1, decoder.clone());
    let consumer2 = manager.register_consumer(2, decoder.clone());
    assert_eq!(decoder.ref_count(), 2);
    
    manager.unregister_consumer(1);
    assert_eq!(decoder.ref_count(), 1);
    
    manager.unregister_consumer(2);
    assert_eq!(decoder.ref_count(), 0);
    // Decoder should be removed from manager
}
```

### Integration Tests

```bash
# Test with 4 outputs showing same video
wayvid test --outputs 4 --same-video --duration 60s --measure-cpu

# Test with mixed scenario
wayvid test --outputs 4 --videos "vid1.mp4,vid1.mp4,vid2.mp4,vid1.mp4"

# Test hotplug
wayvid test --outputs 4 --hotplug-interval 5s --duration 60s
```

### Performance Benchmarks

```bash
# Before/after comparison
cargo bench --bench shared_decode

# Expected results:
#   4 outputs, same video:
#     Before: 40% CPU, 500MB memory
#     After:  16% CPU, 135MB memory
```

## Alternatives Considered

### Alternative 1: MPV Shared Video Output
**Idea**: Use MPV's own multi-output support.

**Pros**: Potentially simpler, less code.

**Cons**: 
- MPV doesn't natively support this
- Would require upstream changes
- Less control over frame delivery

**Decision**: Rejected. Too dependent on upstream.

### Alternative 2: Single MPV, Multiple GL Contexts
**Idea**: One MPV, render to multiple GL contexts.

**Pros**: Simpler decode management.

**Cons**:
- GL contexts can't share textures across Wayland surfaces
- Would need frame copying anyway
- Doesn't reduce render overhead

**Decision**: Rejected. Doesn't solve the core problem.

### Alternative 3: Fork Per Output
**Idea**: Fork process for each output.

**Pros**: Complete isolation.

**Cons**:
- No actual sharing
- Even higher resource usage
- Complex IPC

**Decision**: Rejected. Makes problem worse.

## Open Questions

1. **Thread Safety**: Should frame buffer use lock-free queue?
   - **Answer**: Start with Mutex, profile later. Premature optimization.

2. **Texture Sharing**: Can we share GL textures between surfaces?
   - **Answer**: No, Wayland surfaces have separate GL contexts. Must copy.

3. **Frame Drop Strategy**: What if consumer can't keep up?
   - **Answer**: Skip to latest frame. Consumers are independent.

4. **Config Reload**: Graceful transition or restart decoders?
   - **Answer**: Graceful. Reuse decoders where possible.

## Future Enhancements

1. **Lock-Free Frame Buffer** (M6)
   - Use atomic ring buffer
   - Eliminate mutex contention

2. **Zero-Copy Rendering** (M6)
   - DMA-BUF sharing between MPV and Wayland
   - Requires compositor support

3. **Predictive Decoding** (M6)
   - Decode ahead for consumers
   - Reduce latency spikes

## References

- [MPV Render API](https://github.com/mpv-player/mpv/blob/master/libmpv/render.h)
- [Wayland Book - Surfaces](https://wayland-book.com/)
- [Rust Condvar docs](https://doc.rust-lang.org/std/sync/struct.Condvar.html)

---

## Status Updates

### 2025-10-23: Initial Draft
- Created RFC
- Defined architecture
- Identified key components

### TBD: Review
- Gather feedback
- Refine design
- Approve for implementation

### TBD: Implementation
- Start coding
- Track progress in M5_TODO.md

---

**Approval Status**: ⏳ Awaiting Review  
**Reviewers**: [TBD]  
**Implementation ETA**: Week 1 of M5
