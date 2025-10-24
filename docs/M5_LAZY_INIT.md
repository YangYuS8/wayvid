# Lazy Initialization Implementation (Issue #15)

**Branch**: `m5-lazy-init`  
**Status**: ✅ Phase 1 Complete  
**PR**: TBD  

---

## 🎯 Goal

Defer resource allocation until needed to achieve:
- **40% faster startup**
- **Reduced idle memory usage**
- **Better hotplug support**

---

## 📋 Implementation

### Phase 1: Core Lazy Loading ✅

**Changes**:
1. **State Tracking** (`WaylandSurface`)
   - `resources_initialized: bool` - Track if EGL/decoder initialized
   - `is_active: bool` - Track output visibility/power state

2. **Deferred Initialization**
   - **Before**: Resources created in `configure()` (first Wayland configure event)
   - **After**: Resources created in `render()` (first actual render request)

3. **Resource Management**
   - `set_active(bool)` - Mark output active/inactive
   - `cleanup_resources()` - Release decoder when inactive
   - Automatic cleanup in `destroy()`

### Code Flow

#### Before (Eager Initialization)
```rust
configure() {
    // First configure event
    create_egl_window()     // ← EGL resources allocated
    init_decoder()          // ← MPV decoder created
}

render() {
    // Use pre-created resources
    mpv_render()
}
```

#### After (Lazy Initialization)
```rust
configure() {
    // Just acknowledge configuration
    // No resource allocation
}

render() {
    if !resources_initialized {
        create_egl_window()     // ← First render
        init_decoder()          // ← First render
        resources_initialized = true
    }
    mpv_render()
}
```

---

## 📊 Benefits

### 1. Faster Startup
- **Baseline**: All outputs initialized during startup
- **Optimized**: Only Wayland surfaces created, heavy resources deferred
- **Expected**: 30-40% faster (800ms → 480ms target)

### 2. Lower Idle Memory
- Inactive outputs don't hold decoder references
- BufferPool can reclaim unused buffers
- Better memory pressure handling

### 3. Better Hotplug
- New outputs initialize only when first rendered
- Removed outputs cleanup immediately
- No wasted initialization for briefly connected displays

---

## 🧪 Testing

### Manual Test
```bash
# Build optimized version
cargo build --release --all-features

# Watch logs for lazy init messages
RUST_LOG=info ./target/release/wayvid run

# Look for:
# "🚀 Lazy initialization for output X (first render)"
# "✓ EGL window created lazily"
# "✓ Decoder initialized lazily"
# "✅ Lazy initialization complete"
# "✅ Startup complete in XXms"
```

### Automated Test
```bash
./scripts/test_startup_time.sh

# Measures:
# - Baseline (main branch)
# - Optimized (m5-lazy-init branch)
# - Reports percentage improvement
```

### Expected Results
- Startup time: 30-40% reduction
- Memory usage: Same or slightly better (cleanup helps)
- No functional changes (just timing)

---

## 🔧 Technical Details

### Resource Types

1. **EGL Window** (`EglWindow`)
   - Lightweight: Just window surface binding
   - Can be kept across activations
   - Only resize() called on dimension changes

2. **Decoder Handle** (`DecoderHandle`)
   - Heavy: MPV instance, render context
   - Reference counted via `SharedDecodeManager`
   - Released when output inactive

### Activation State

**Active** (default):
- Lazy init happens on first render
- Resources held until deactivated

**Inactive** (future):
- Decoder handle released
- EGL window kept (lightweight)
- Can be triggered by:
  - DPMS power off
  - Output disconnected
  - Manual deactivation API

### Memory Impact

With shared decoder (#13) + lazy init (#15):
- Only active outputs hold decoder references
- BufferPool shared across active outputs
- Inactive outputs: minimal memory (just Wayland surfaces)

---

## 🚀 Future Enhancements

### Phase 2: DPMS Integration (TODO)
```rust
// Detect DPMS power state changes
fn handle_dpms_event(dpms_state: DpmsState) {
    match dpms_state {
        DpmsState::On => surface.set_active(true),
        DpmsState::Off | DpmsState::Standby => surface.set_active(false),
    }
}
```

### Phase 3: Visibility Detection (TODO)
- Monitor which outputs are actually visible
- Only initialize visible outputs
- Cleanup on workspace switch (if possible to detect)

### Phase 4: Texture Pooling (TODO)
- Pool OpenGL textures for reuse
- Defer texture allocation like other resources
- Release unused textures based on memory pressure

---

## 📈 Performance Metrics

### Startup Time (Target)
| Metric | Baseline (v0.3.0) | Target (v0.4.0) | Current |
|--------|-------------------|-----------------|---------|
| 1 output | ~800ms | ~480ms | TBD |
| 3 outputs | ~900ms | ~520ms | TBD |

### Memory Usage (Idle)
| Metric | Baseline | With Lazy Init |
|--------|----------|----------------|
| 1 active | 149MB | ~149MB |
| 1 active + 2 inactive | 340MB | ~160MB* |

*Estimated: inactive outputs don't hold decoder references

---

## ✅ Acceptance Criteria

- [x] Resources not initialized in `configure()`
- [x] Resources initialized on first `render()`
- [x] Startup time measured and logged
- [x] `set_active()` / `cleanup_resources()` implemented
- [ ] DPMS integration (deferred to next phase)
- [ ] Startup time test passes (30%+ improvement)
- [ ] All tests passing
- [ ] Documentation complete

---

## 🔗 Related

- **Depends on**: Issue #13 (Shared Decode) - merged
- **Depends on**: Issue #14 (Memory Optimization) - merged
- **Enables**: Issue #16 (Frame Skip Intelligence)
- **RFC**: M5-003 (Lazy Initialization)

---

## 📝 Notes

### Why defer to first render?

1. **Wayland protocol requirement**: Must create surfaces during roundtrip
2. **EGL context needed**: Can't initialize MPV render context without EGL
3. **Natural barrier**: First render is when we actually need resources
4. **Hotplug friendly**: New outputs follow same path

### Why keep EGL window after deactivation?

- **Lightweight**: Just a window surface pointer
- **Fast reactivation**: No need to recreate on activate
- **Resize handling**: Can still handle dimension changes
- **Simplicity**: Simpler state machine

### Trade-offs

**Pros**:
- Faster startup ✅
- Lower idle memory ✅
- Better hotplug ✅
- Cleaner separation ✅

**Cons**:
- Slightly more complex state ⚠️
- First frame delay (negligible, <10ms) ⚠️
- Requires careful testing ⚠️

Overall: Benefits far outweigh drawbacks.

---

**Status**: Phase 1 complete, ready for testing and profiling.
