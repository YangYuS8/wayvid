# M5 Quick Reference Guide

## 🎯 What is M5?

**M5 (Milestone 5)** focuses on **Performance & Polish**:
- Make wayvid faster and more efficient
- Add advanced features (HDR, playlists)
- Improve user experience
- Expand distribution support

**Target**: v0.4.0  
**Duration**: 3-4 weeks  
**Start**: TBD

---

## 📚 Documentation Structure

```
M5_PLAN.md              # Full planning document (1453 lines)
├── Overview & goals
├── Feature breakdown
├── Timeline & sprints
├── Testing strategy
└── Success metrics

M5_TODO.md              # Task list with time estimates
├── Phase 1: Performance (50h)
├── Phase 2: Features (45h)
├── Phase 3: Polish (45h)
├── Phase 4: Distribution (37h)
└── Daily progress tracking

docs/rfcs/
└── M5-001-shared-decode.md   # Technical design for shared decode
    ├── Architecture
    ├── API design
    ├── Performance analysis
    └── Implementation plan
```

---

## 🎯 Key Features (TL;DR)

### P0 - Critical (Week 1)
- **Shared Decode**: Same video on multiple outputs → 60% less CPU
- **Memory Optimization**: Frame buffer pooling → 40% less memory
- **Lazy Init**: Don't load unused outputs → faster startup
- **Frame Skip**: Adaptive FPS → smoother under load

### P1 - High Priority (Week 2)
- **HDR Support**: HDR10/HLG passthrough + tone mapping
- **Multi-Monitor**: Per-output video sources, patterns
- **Playlists**: Directory sources with rotation
- **Audio Reactive**: Basic FFT for audio-reactive effects

### P2 - Medium Priority (Week 2-3)
- **Error Handling**: Desktop notifications, recovery
- **Config Validator**: Check configs before apply
- **Setup Wizard**: Interactive first-run setup
- **Diagnostics**: Performance stats, frame timing

### P3 - Distribution (Week 3)
- **Debian/Ubuntu**: PPA packages
- **Fedora**: COPR packages
- **Flatpak**: Flathub submission
- **ARM64**: Raspberry Pi support

---

## 📊 Performance Targets

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| CPU (4 outputs, same video) | ~44% | ~16% | **-64%** |
| Memory (4 outputs) | ~500MB | ~135MB | **-73%** |
| Startup time | ~2s | <500ms | **-75%** |
| Config reload | ~300ms | <100ms | **-67%** |

---

## 🏃 How to Start M5

### 1. Pre-Development
```bash
# Update development branch
git checkout main
git pull

# Create M5 feature branch
git checkout -b m5-performance

# Review planning docs
cat M5_PLAN.md | less
cat M5_TODO.md | less

# Review RFC for shared decode
cat docs/rfcs/M5-001-shared-decode.md | less
```

### 2. Setup Tracking
```bash
# Create GitHub project (if not exists)
gh project create "M5 - Performance & Polish"

# Add issues from M5_TODO.md
# (Manual: Convert tasks to GitHub issues)

# Start daily log
echo "## Day 1 - $(date)" >> M5_PROGRESS.md
```

### 3. Begin Implementation
```bash
# Start with highest priority
# Week 1: Shared decode (see RFC)

# Create module
mkdir -p src/video/shared_decode
touch src/video/shared_decode/{mod.rs,manager.rs,decoder.rs,consumer.rs}

# Follow RFC architecture
# Implement tests as you go
# Update M5_TODO.md daily
```

---

## 📝 Daily Workflow

### Morning
1. Review yesterday's progress
2. Update M5_TODO.md with completed tasks
3. Choose today's tasks from sprint plan
4. Write daily goal in M5_PROGRESS.md

### During Work
1. Follow TDD: Write test first
2. Implement feature
3. Run tests: `cargo test`
4. Check performance: `cargo bench` (if applicable)
5. Commit frequently with good messages

### Evening
1. Run full test suite: `cargo test --all`
2. Check CI status: `gh run list`
3. Update progress tracking
4. Plan tomorrow's tasks
5. Push changes

---

## 🧪 Testing Commands

### Unit Tests
```bash
# All tests
cargo test --all

# Specific module
cargo test shared_decode

# With output
cargo test -- --nocapture

# Single test
cargo test test_shared_decode_manager
```

### Integration Tests
```bash
# Performance test (when implemented)
cargo run --release -- test --outputs 4 --duration 60s

# Benchmark
cargo bench --bench shared_decode

# Memory leak check
valgrind --leak-check=full target/debug/wayvid
```

### CI Check
```bash
# Before pushing
cargo check --all-features
cargo clippy --all-features
cargo fmt -- --check
cargo test --all
```

---

## 📊 Progress Tracking

### Update M5_TODO.md
```markdown
### Day 1 - Shared Decode Architecture

**Goals**:
- [x] Design SharedDecodeManager API
- [x] Implement SourceKey hashing
- [ ] Create SharedDecoder skeleton

**Blockers**: None

**Notes**: SourceKey design reviewed, looks good

**Tomorrow**: Finish SharedDecoder, start Consumer
```

### Update M5_PROGRESS.md (Create if needed)
```markdown
## Week 1 Progress

### Monday (Day 1)
- Designed SharedDecodeManager
- Implemented SourceKey with proper hashing
- Wrote RFC review notes
- Hours: 6/8

### Tuesday (Day 2)
- ...
```

---

## 🔍 Key Files to Edit

### Phase 1: Performance
```
src/video/
├── shared_decode/         # NEW: Shared decode system
│   ├── mod.rs
│   ├── manager.rs         # SharedDecodeManager
│   ├── decoder.rs         # SharedDecoder
│   └── consumer.rs        # Consumer
├── mpv.rs                 # UPDATE: Refactor for sharing
└── egl.rs                 # UPDATE: Frame buffer management

src/backend/wayland/
└── surface.rs             # UPDATE: Use Consumer instead of direct MPV
```

### Phase 2: Features
```
src/
├── hdr/                   # NEW: HDR support
│   ├── mod.rs
│   ├── detection.rs       # HDR content detection
│   └── tone_mapping.rs    # SDR tone mapping
├── playlist/              # NEW: Playlist management
│   ├── mod.rs
│   └── manager.rs
└── audio/                 # NEW: Audio reactivity
    ├── mod.rs
    └── fft.rs

src/config/
└── types.rs               # UPDATE: Add per_output sources
```

### Phase 3: Polish
```
src/
├── error/                 # NEW: Better error handling
│   ├── mod.rs
│   ├── notification.rs    # Desktop notifications
│   └── recovery.rs        # Recovery strategies
└── diagnostics/           # NEW: Diagnostic tools
    ├── mod.rs
    └── stats.rs

src/bin/
└── wayvid-ctl.rs          # UPDATE: Add health, stats commands
```

---

## 💡 Tips & Best Practices

### Code Quality
- Write tests BEFORE implementation (TDD)
- Use `#[cfg(test)]` for test modules
- Add rustdoc comments for public APIs
- Run `cargo clippy` frequently
- Keep functions under 50 lines

### Performance
- Use `cargo flamegraph` to find bottlenecks
- Benchmark before and after optimizations
- Profile with `perf` on Linux
- Use `cargo-bloat` to check binary size

### Git Workflow
- Commit frequently (every feature/fix)
- Use conventional commits:
  - `feat: Add shared decode manager`
  - `perf: Optimize frame buffer allocation`
  - `test: Add integration tests for hotplug`
  - `docs: Update performance metrics`
- Push at end of day (or more often)

### Communication
- Update GitHub issues with progress
- Ask questions in Discussions
- Document blockers immediately
- Celebrate wins! 🎉

---

## 📞 Resources

### Documentation
- **Full Plan**: `M5_PLAN.md` - Read first
- **Tasks**: `M5_TODO.md` - Daily reference
- **RFC**: `docs/rfcs/M5-001-shared-decode.md` - Technical details

### External Resources
- [MPV Render API](https://mpv.io/manual/stable/#embedding-into-other-programs-libmpv)
- [Wayland Protocols](https://wayland.app/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

### Tools
- **Profiling**: `cargo flamegraph`
- **Benchmarking**: `cargo bench`
- **Memory**: `valgrind`, `heaptrack`
- **CI**: GitHub Actions

---

## 🚦 Checkpoints

### End of Week 1
- [ ] All P0 features implemented
- [ ] Performance targets met (60% CPU reduction)
- [ ] Tests passing
- [ ] RFC updated with findings

### End of Week 2
- [ ] All P1 features implemented
- [ ] HDR working on test setup
- [ ] Playlist functional
- [ ] Documentation updated

### End of Week 3
- [ ] All P2 features implemented
- [ ] Error handling working
- [ ] Diagnostic tools functional
- [ ] Distribution packages ready

### End of Week 4 (Release)
- [ ] Final testing complete
- [ ] Documentation finalized
- [ ] CHANGELOG updated
- [ ] v0.4.0 tagged and released
- [ ] Packages published

---

## 🎉 Success Criteria

M5 is **complete** when:
- ✅ All P0 and P1 features working
- ✅ Performance targets met
- ✅ All tests passing (70%+ coverage)
- ✅ CI green
- ✅ Documentation complete
- ✅ v0.4.0 released
- ✅ Packages on 3+ repos

---

## ❓ FAQ

**Q: Where do I start?**  
A: Read M5_PLAN.md, then M5_TODO.md, then begin with shared decode (Week 1, Day 1)

**Q: How detailed should commits be?**  
A: Each commit should be one logical change with good message. Reference issues.

**Q: What if I get stuck?**  
A: Document the blocker in M5_PROGRESS.md, ask in GitHub Discussions, or skip to next task.

**Q: How often should I test?**  
A: Run `cargo test` after every feature. Full suite before pushing.

**Q: Should I follow the plan exactly?**  
A: Plan is a guide, not law. Adapt based on findings. Document changes.

**Q: When is M5 done?**  
A: When all success criteria are met and v0.4.0 is released.

---

**Status**: 📋 Ready to Start  
**Next Action**: Review M5_PLAN.md and create feature branch  
**ETA**: 3-4 weeks from start date
