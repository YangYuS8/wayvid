# Development Notes - M1 MVP Completion

## Current Status (2025-10-20)

### What's Implemented ✅

#### Project Infrastructure
- ✅ Cargo project with proper feature flags
- ✅ Module structure (core, backend, video, ctl)
- ✅ Configuration system (YAML with per-output overrides)
- ✅ CLI with clap (run, check commands)
- ✅ Logging with tracing
- ✅ Error handling with anyhow
- ✅ CI pipeline (GitHub Actions)
- ✅ Packaging scaffolds (AUR, Nix, systemd)
- ✅ Comprehensive documentation

#### Wayland Backend
- ✅ Connection to compositor
- ✅ Registry and global discovery
- ✅ wlr-layer-shell surface creation
- ✅ Background layer placement
- ✅ Input passthrough configuration
- ✅ Output (monitor) tracking
- ✅ Basic event loop with Dispatch traits
- ⚠️ EGL context setup (placeholder)
- ⚠️ Frame callbacks (simplified)

#### Video Playback
- ✅ libmpv initialization
- ✅ Playback options (loop, speed, start time, volume)
- ✅ Hardware decode configuration
- ✅ Video file loading
- ⚠️ Rendering (simplified, no actual GL output)

#### Layout System
- ✅ Layout mode types (Fill, Contain, Stretch, Centre)
- ✅ Layout calculation algorithms
- ✅ Unit tests for layout math

#### Capability Checking
- ✅ Wayland connection check
- ✅ Protocol availability check
- ✅ Video backend verification
- ✅ Hardware decode check
- ✅ System library check

### What's NOT Implemented (Known Gaps)

#### Critical for M2
1. **OpenGL/EGL Rendering Pipeline** ❗
   - Current: Uses null video output
   - Needed: mpv_render_context integration
   - Needed: FBO rendering to layer surface
   - Needed: Proper EGL context management with wl_egl_window
   
2. **Frame Synchronization**
   - Current: No vsync
   - Needed: Frame callback handling
   - Needed: wp_presentation protocol (optional)
   - Needed: Render throttling

3. **Multi-Output Hotplug**
   - Current: Static output discovery
   - Needed: Output add/remove events
   - Needed: Dynamic surface creation/destruction
   - Needed: xdg-output protocol for names

4. **Power Management**
   - Current: Config parsed but not enforced
   - Needed: DPMS state tracking
   - Needed: Idle inhibit protocol
   - Needed: Battery state detection
   - Needed: FPS limiting

#### Important for M3
5. **Wallpaper Engine Import**
   - Needed: project.json parser
   - Needed: Parameter mapping
   - Needed: Config generator

6. **Distribution Packages**
   - Partial: AUR PKGBUILD ready
   - Partial: Nix flake ready
   - Needed: AppImage build script
   - Needed: Flatpak manifest
   - Needed: Debian/RPM packaging

## Code Quality Notes

### What's Good
- Clean module separation
- Type-safe configuration
- Comprehensive error handling
- Good documentation coverage
- Test framework in place

### What Needs Improvement
- More unit tests (currently minimal)
- Integration tests require manual setup
- Some dead code (prepared for future)
- EGL/OpenGL code is placeholder
- Frame timing is non-existent

## Technical Debt

### High Priority
1. **EGL Context Management**
   - Current implementation is unsafe
   - Need proper wl_egl_window integration
   - Need error recovery for GL failures

2. **Memory Safety**
   - Wayland object lifetimes need review
   - Surface/player lifecycle needs careful testing
   - Potential race conditions in event loop

3. **Resource Cleanup**
   - Ensure proper Drop implementations
   - Test hotplug scenarios thoroughly
   - Handle compositor crashes gracefully

### Medium Priority
4. **Configuration Validation**
   - Need schema validation
   - Better error messages for invalid configs
   - Default value documentation

5. **Performance Profiling**
   - No profiling done yet
   - Memory usage unknown at scale
   - CPU usage needs measurement

## Known Issues

### Compilation
- ⚠️ 10 warnings about unused code (expected, for future features)
- ✅ No errors, compiles successfully

### Runtime (Untested)
- ❌ Will not display video (null vo in mpv)
- ❌ No actual rendering to screen
- ✅ Layer surface should be created
- ✅ Input passthrough should work
- ? Multi-output behavior unknown

### Compatibility
- ✅ Should work on Hyprland (untested)
- ✅ Should work on niri (untested)
- ❓ Other wlr compositors (probably works)
- ❌ Non-wlr compositors (not supported)

## Testing Checklist for M1 Review

### Build Tests
- [x] `cargo build` succeeds
- [x] `cargo build --release` succeeds
- [ ] `cargo test` passes (need Wayland)
- [x] `cargo clippy` passes (with warnings)
- [x] `cargo fmt --check` passes

### Runtime Tests (TODO)
- [ ] `wayvid check` shows system info
- [ ] `wayvid run` creates layer surface
- [ ] Input passthrough works (can click through)
- [ ] Surface appears on all outputs
- [ ] Config loading works
- [ ] Per-output overrides work
- [ ] Graceful shutdown (Ctrl+C)

### Compositor Tests (TODO)
- [ ] Works on Hyprland
- [ ] Works on niri
- [ ] Works on Sway
- [ ] Survives compositor restart

## Migration Path to M2

### Phase 1: OpenGL Integration
1. Replace EGL placeholder with real implementation
2. Create wl_egl_window for each surface
3. Initialize mpv_render_context with OpenGL
4. Implement FBO rendering
5. Test basic video display

### Phase 2: Frame Timing
1. Implement frame callback handling
2. Add render throttling
3. Integrate with mpv event loop
4. Test smooth playback

### Phase 3: Multi-Output
1. Add xdg-output protocol support
2. Implement output add/remove handlers
3. Test hotplug scenarios
4. Optimize for multiple outputs

### Phase 4: Power Management
1. Track DPMS state per output
2. Implement pause/resume logic
3. Add battery detection
4. Implement FPS limiter

## Dependencies Review

### Critical Dependencies
- ✅ wayland-client 0.31 (stable)
- ✅ wayland-protocols 0.32 (stable)
- ✅ wayland-protocols-wlr 0.3 (stable)
- ✅ smithay-client-toolkit 0.19 (stable)
- ✅ libmpv 2.0 (stable)
- ✅ khronos-egl 6.0 (stable)
- ✅ gl 0.14 (stable)

### Potential Issues
- None identified yet
- May need mpv-sys or direct C bindings for render context

## Performance Expectations (Estimated)

### M1 MVP (Current)
- CPU: Unknown (null vo, minimal)
- Memory: ~100MB (mpv instance)
- GPU: 0% (no rendering)

### M2 Target
- CPU: 2-5% (with hwdec)
- CPU: 10-20% (software decode)
- Memory: 100-300MB per output
- GPU: 5-15% (decode + render)

## Next Steps (Immediate)

1. **Test Current Build**
   - Run on Hyprland/niri
   - Verify layer surface creation
   - Check input passthrough
   - Confirm config loading

2. **Begin OpenGL Work**
   - Research mpv_render_context API
   - Study wl_egl_window creation
   - Prototype FBO rendering
   - Test with simple video

3. **Document Findings**
   - Update known issues
   - Document any bugs found
   - Create issues for M2 work
   - Update roadmap

## Questions for Review

1. Is the module structure appropriate?
2. Should we use a different video backend?
3. Is error handling too verbose?
4. Should we support X11 eventually?
5. Any security concerns with file paths?

## Resources for M2 Development

### Essential Reading
- [mpv client API](https://mpv.io/manual/master/#client-api)
- [mpv render context](https://mpv.io/manual/master/#render-c-api)
- [wlr-layer-shell protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1)
- [Wayland book](https://wayland-book.com/)

### Example Projects
- [mpv-android](https://github.com/mpv-android/mpv-android) - mpv OpenGL integration
- [lavalauncher](https://git.sr.ht/~leon_plickat/lavalauncher) - Layer shell example
- [waybar](https://github.com/Alexays/Waybar) - Complex layer shell app

### Tools
- `wayland-info` - Inspect Wayland protocols
- `vainfo` / `vdpauinfo` - Check hardware decode
- `glxinfo` - Check OpenGL capabilities
- `mpv --vo=gpu --log-file=mpv.log` - Test video playback

---

**Conclusion**: M1 MVP skeleton is complete and compiles successfully. The architecture is sound and ready for M2 implementation. The main work ahead is OpenGL/EGL integration and frame synchronization.
