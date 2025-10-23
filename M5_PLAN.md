# M5 Milestone Planning - Performance & Polish

## 📋 Overview
**Target Version**: v0.4.0  
**Theme**: Performance Optimization & User Experience  
**Estimated Duration**: 2-3 weeks  
**Priority**: High-impact features for production use

---

## 🎯 Goals

### Primary Objectives
1. **Performance Optimization** - Reduce CPU/GPU/Memory usage
2. **Advanced Features** - Shared decode, HDR support, better multi-monitor
3. **User Experience** - Better defaults, error handling, diagnostics
4. **Platform Support** - More distributions, better compatibility

### Success Metrics
- 50% reduction in CPU usage for multi-output scenarios
- HDR content playback on supported hardware
- Sub-100ms configuration reload time
- Zero crashes in 24-hour stress test
- Package availability on 5+ distributions

---

## 🚀 Feature Breakdown

### Phase 1: Performance Optimization (Week 1)
**Priority**: Critical  
**Estimated Effort**: 5-7 days

#### 1.1 Shared Decode Context
**Problem**: Currently each output creates its own MPV instance, duplicating decode work for same video.

**Implementation**:
```rust
// New module: src/video/shared_decode.rs
pub struct SharedDecodeManager {
    decoders: HashMap<String, Arc<Mutex<MpvPlayer>>>,
    consumers: HashMap<u32, WeakRef<Consumer>>,
}

pub struct Consumer {
    output_id: u32,
    frame_buffer: Arc<Mutex<FrameBuffer>>,
}
```

**Tasks**:
- [ ] Design shared decode architecture
- [ ] Implement decode manager with reference counting
- [ ] Add consumer registration/unregistration
- [ ] Implement frame buffer sharing
- [ ] Add configuration option `shared_decode: bool`
- [ ] Test with 2-4 outputs showing same video
- [ ] Benchmark CPU usage reduction

**Expected Benefit**: 60-70% CPU reduction when showing same video on multiple outputs

#### 1.2 Memory Optimization
**Problem**: MPV render contexts can use significant memory, especially with high-res videos.

**Tasks**:
- [ ] Add memory usage tracking
- [ ] Implement frame buffer pooling
- [ ] Add texture cache eviction
- [ ] Optimize EGL surface allocation
- [ ] Add `max_memory_mb` configuration option
- [ ] Log memory statistics on debug builds

**Expected Benefit**: 30-40% memory reduction

#### 1.3 Lazy Initialization
**Problem**: All outputs initialize immediately, even if hidden/powered off.

**Tasks**:
- [ ] Implement lazy surface creation
- [ ] Defer MPV init until output is active
- [ ] Add output visibility detection
- [ ] Add DPMS state awareness
- [ ] Test hotplug scenarios

**Expected Benefit**: Faster startup, lower idle resource usage

#### 1.4 Frame Skip Intelligence
**Problem**: Current frame callback doesn't adapt to load.

**Tasks**:
- [ ] Implement frame skip heuristics
- [ ] Add GPU load detection (if available)
- [ ] Dynamic FPS adjustment based on power profile
- [ ] Add `target_fps` and `min_fps` config options
- [ ] Implement presentation time tracking

**Expected Benefit**: Smoother playback under load

---

### Phase 2: Advanced Features (Week 2)
**Priority**: High  
**Estimated Effort**: 6-8 days

#### 2.1 HDR Support
**Problem**: HDR videos play in SDR, losing color/brightness range.

**Tasks**:
- [ ] Detect HDR10/HLG content from MPV properties
- [ ] Query output HDR capabilities via Wayland protocols
- [ ] Configure MPV for HDR passthrough
- [ ] Add HDR tone mapping for SDR displays
- [ ] Add configuration: `hdr_mode: auto|force|disable`
- [ ] Test with HDR test videos

**Dependencies**: Compositor HDR support (Hyprland 0.40+)

#### 2.2 Advanced Multi-Monitor
**Problem**: Limited per-output control, no monitor-specific WE projects.

**Tasks**:
- [ ] Add per-output video source override
- [ ] Support different WE projects per output
- [ ] Add output name patterns (e.g., `HDMI-*`, `DP-1`)
- [ ] Implement output priority/fallback
- [ ] Add `wayvid-ctl set-output-source <name> <path>` command

**Configuration Example**:
```yaml
per_output:
  "DP-1":  # Left monitor
    source:
      type: WeProject
      path: "~/wallpapers/cyberpunk/"
  "HDMI-*":  # Any HDMI output
    source:
      type: File
      path: "~/wallpapers/nature.mp4"
```

#### 2.3 Audio Reactivity (Basic)
**Problem**: Some WE wallpapers react to audio, wayvid doesn't support this.

**Tasks**:
- [ ] Add PulseAudio/PipeWire FFT capture
- [ ] Expose audio spectrum as MPV properties
- [ ] Add Lua scripting support for audio-reactive filters
- [ ] Add `audio_reactive: bool` config option
- [ ] Document Lua API for custom effects

**Scope**: Basic FFT data only, no full WE audio processing

#### 2.4 Playlist Support
**Problem**: Can only play one video per output.

**Tasks**:
- [ ] Support VideoSource::Directory with shuffle/sequential
- [ ] Add playlist rotation interval
- [ ] Implement cross-fade transitions
- [ ] Add `wayvid-ctl next/prev` commands
- [ ] Support M3U/PLS playlist files

**Configuration Example**:
```yaml
source:
  type: Directory
  path: "~/wallpapers/collection/"
  shuffle: true
  interval: 300  # seconds
  transition: crossfade
```

---

### Phase 3: User Experience (Week 2-3)
**Priority**: Medium  
**Estimated Effort**: 4-5 days

#### 3.1 Better Error Handling
**Problem**: Errors are logged but user has no visibility.

**Tasks**:
- [ ] Add error notification via desktop notification (libnotify)
- [ ] Create error recovery strategies (retry, fallback source)
- [ ] Add `wayvid-ctl health` command
- [ ] Implement graceful degradation (disable failing outputs)
- [ ] Add error codes and troubleshooting docs

#### 3.2 Configuration Validator
**Problem**: Invalid configs cause runtime errors.

**Tasks**:
- [ ] Add `wayvid check-config <file>` command
- [ ] Validate before applying hot-reload
- [ ] Check file existence, permissions
- [ ] Validate hardware capabilities
- [ ] Suggest fixes for common mistakes

#### 3.3 Interactive Setup Wizard
**Problem**: First-run experience is confusing.

**Tasks**:
- [ ] Add `wayvid setup` interactive command
- [ ] Detect compositor and capabilities
- [ ] Generate optimal config based on hardware
- [ ] Test video sources
- [ ] Create systemd/autostart entries

**Example Flow**:
```
$ wayvid setup
🔍 Detecting compositor... Hyprland v0.42.0 ✓
🔍 Checking hardware decode... VA-API (Intel) ✓
🔍 Scanning for video files...

Found 3 video wallpapers:
  1. ~/Videos/ocean.mp4 (1920x1080, 60fps)
  2. ~/Videos/space.mp4 (3840x2160, 30fps)
  3. ~/Downloads/abstract.webm (2560x1440, 24fps)

Select default wallpaper [1]: 
```

#### 3.4 Diagnostic Tools
**Problem**: Hard to troubleshoot performance issues.

**Tasks**:
- [ ] Add `wayvid-ctl stats` command (FPS, CPU, GPU, memory)
- [ ] Implement performance overlay (debug mode)
- [ ] Add frame timing histogram
- [ ] Log MPV properties on demand
- [ ] Export stats to JSON for analysis

---

### Phase 4: Distribution & Compatibility (Week 3)
**Priority**: Medium-High  
**Estimated Effort**: 3-4 days

#### 4.1 Debian/Ubuntu Packages
**Tasks**:
- [ ] Create debian/ directory structure
- [ ] Write control file, rules, changelog
- [ ] Add to PPA (personal package archive)
- [ ] Test on Ubuntu 22.04, 24.04, Debian 12
- [ ] Add installation docs

#### 4.2 Fedora/RPM Packaging
**Tasks**:
- [ ] Create wayvid.spec file
- [ ] Submit to COPR (Fedora user repos)
- [ ] Test on Fedora 39, 40
- [ ] Add installation docs

#### 4.3 Flatpak
**Tasks**:
- [ ] Create org.github.YangYuS8.wayvid.yaml manifest
- [ ] Handle Wayland socket permissions
- [ ] Test on Flathub sandbox
- [ ] Submit to Flathub

#### 4.4 Cross-Compilation
**Tasks**:
- [ ] Add aarch64 (ARM64) CI build
- [ ] Setup cross-compilation toolchain
- [ ] Test on Raspberry Pi 4/5
- [ ] Create ARM64 AppImage

---

## 📊 Technical Debt & Refactoring

### Code Quality Improvements
- [ ] Increase test coverage to 70%+ (currently ~40%)
- [ ] Add integration tests with mock Wayland server
- [ ] Refactor surface.rs (currently 398 lines)
- [ ] Extract EGL/OpenGL code to separate crate
- [ ] Add fuzzing for config parser
- [ ] Document all public APIs with rustdoc

### CI/CD Enhancements
- [ ] Add performance regression tests
- [ ] Implement automated benchmarking
- [ ] Add memory leak detection (Valgrind)
- [ ] Create nightly builds
- [ ] Add changelog auto-generation

---

## 🔧 Implementation Details

### Priority Rankings
1. **P0 (Critical)**: Shared decode, Memory optimization, Better error handling
2. **P1 (High)**: HDR support, Playlist, Config validator
3. **P2 (Medium)**: Audio reactivity, Diagnostic tools, Debian/Fedora packages
4. **P3 (Nice to have)**: Interactive setup, Flatpak, ARM64 support

### Risk Assessment

| Feature | Risk Level | Mitigation |
|---------|------------|------------|
| Shared Decode | High | Extensive testing, fallback to single decode |
| HDR Support | Medium | Compositor-dependent, graceful degradation |
| Audio Reactivity | Medium | Optional feature, disable on errors |
| Flatpak | Low | Sandbox restrictions documented |

### Dependencies
- **External**: 
  - Compositor HDR support (Hyprland 0.40+, niri pending)
  - PulseAudio/PipeWire for audio reactivity
  - Wayland presentation-time protocol
- **Internal**:
  - Shared decode requires refactoring MPV wrapper
  - HDR requires EGL context changes

---

## 📅 Timeline

### Week 1: Foundation
- Days 1-2: Shared decode architecture design & implementation
- Days 3-4: Memory optimization & lazy initialization
- Days 5-7: Frame skip intelligence & testing

### Week 2: Features
- Days 1-2: HDR support implementation
- Days 3-4: Advanced multi-monitor features
- Days 5-6: Playlist support
- Day 7: Audio reactivity (basic)

### Week 3: Polish & Distribution
- Days 1-2: Error handling & config validator
- Days 3-4: Diagnostic tools & setup wizard
- Days 5-7: Debian/Fedora packages, documentation

---

## 🧪 Testing Strategy

### Performance Tests
```bash
# Test shared decode with 4 outputs
wayvid test --outputs 4 --shared-decode --duration 60s

# Memory leak test
valgrind --leak-check=full wayvid run --config test.yaml

# Benchmark mode
wayvid benchmark --video test.mp4 --outputs 1,2,4 --duration 30s
```

### Integration Tests
- Hotplug 10 outputs in 10 seconds
- Config reload 100 times consecutively
- Play 100 different videos in sequence
- 24-hour stress test with random events

### Compatibility Tests
- Test on Hyprland 0.40, 0.41, 0.42
- Test on niri (latest git)
- Test on Sway 1.9+
- Test with different GPUs (Intel, AMD, NVIDIA)

---

## 📝 Documentation Updates

### New Docs Needed
- [ ] Performance tuning guide
- [ ] HDR setup guide
- [ ] Troubleshooting flowchart
- [ ] Lua scripting API reference
- [ ] Distribution-specific install guides

### Updated Docs
- [ ] README: Add performance specs
- [ ] CONFIGURATION.md: Document all new options
- [ ] DEV_NOTES.md: Update M5 completion status

---

## 🎓 Lessons from M4

### What Worked Well
- Iterative CI fixes caught issues early
- Good documentation helped onboarding
- Feature flags allowed incremental development

### What to Improve
- Need better local testing before CI push
- More unit tests for complex logic
- Earlier community feedback on features

### Applied to M5
- Write tests first for performance features
- Set up local CI runner for faster iteration
- Create GitHub Discussions for feature proposals
- Weekly progress updates

---

## 🚦 Release Criteria

### Minimum Requirements (v0.4.0)
- ✅ All P0 features implemented
- ✅ 70%+ test coverage
- ✅ CI passing on all platforms
- ✅ Zero known crashes
- ✅ Documentation complete
- ✅ Performance benchmarks improved by 40%+

### Nice to Have
- 🟡 All P1 features implemented
- 🟡 Debian/Fedora packages available
- 🟡 Flatpak submitted to Flathub
- 🟡 Community contributions merged

---

## 📈 Success Metrics (v0.4.0)

### Performance KPIs
- CPU usage: <5% per output (single video, shared decode)
- Memory: <200MB for 4 outputs (1080p video)
- Startup time: <500ms cold start
- Config reload: <100ms

### Quality KPIs
- Test coverage: 70%+
- Clippy warnings: 0
- Documentation: 100% public APIs
- Issues closed: 90%+ within 7 days

### Adoption KPIs
- GitHub stars: 500+ (currently ~50)
- AUR votes: 50+ (currently ~5)
- Downloads: 1000+ per month

---

## 🤝 Community Engagement

### Contribution Opportunities
- Performance testing on different hardware
- HDR testing with various displays
- Distribution packaging volunteers
- Documentation translations
- Lua script examples

### Communication
- GitHub Discussions for feature proposals
- Matrix/Discord for community support
- Monthly dev blogs on progress

---

## 🔮 Looking Ahead to M6

### Potential M6 Features (v0.5.0)
1. **Full WE Compatibility**
   - Interactive wallpapers
   - Mouse/keyboard input
   - Audio processing (advanced)

2. **Cloud Integration**
   - Wallpaper Engine Workshop browser
   - One-click download and install
   - Automatic updates

3. **GUI Configuration**
   - GTK/Qt settings app
   - Live preview
   - Visual editor

4. **Plugin System**
   - Third-party effect plugins
   - Custom shaders
   - API for external control

---

## ✅ Checklist

### Planning Phase
- [x] Review M4 completion status
- [x] Analyze current codebase for bottlenecks
- [x] Research community needs
- [x] Define success metrics
- [x] Create detailed feature breakdown
- [x] Estimate effort and timeline
- [x] Identify risks and dependencies

### Pre-Development
- [ ] Create GitHub project board for M5
- [ ] Set up performance benchmarking infrastructure
- [ ] Create feature branches for each major feature
- [ ] Write RFCs for complex features (shared decode, HDR)
- [ ] Set up test environment for multi-output scenarios

### Development (TBD)
- [ ] Implement P0 features
- [ ] Implement P1 features
- [ ] Write tests for all new code
- [ ] Update documentation
- [ ] Performance profiling and optimization

### Release (TBD)
- [ ] Final testing and bug fixes
- [ ] Update CHANGELOG
- [ ] Tag v0.4.0
- [ ] Build and publish packages
- [ ] Announce release

---

## 📞 Contact & Feedback

For questions, suggestions, or contributions:
- GitHub Issues: Technical bugs and feature requests
- GitHub Discussions: General questions and ideas
- Email: [maintainer email]

---

**Document Version**: 1.0  
**Created**: 2025-10-23  
**Last Updated**: 2025-10-23  
**Status**: 📋 **PLANNING** → Next: RFC & Implementation
