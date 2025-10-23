# M5 Milestone TODO List

## 🎯 Phase 1: Performance (Week 1) - P0

### Shared Decode Context
- [ ] Design architecture (2h)
- [ ] Implement SharedDecodeManager (4h)
- [ ] Add Consumer registration (2h)
- [ ] Implement frame buffer sharing (4h)
- [ ] Add config option `shared_decode` (1h)
- [ ] Write unit tests (3h)
- [ ] Benchmark and validate (2h)

### Memory Optimization
- [ ] Add memory tracking (2h)
- [ ] Implement frame buffer pooling (3h)
- [ ] Add texture cache eviction (2h)
- [ ] Optimize EGL allocation (2h)
- [ ] Add `max_memory_mb` config (1h)
- [ ] Test and measure reduction (2h)

### Lazy Initialization
- [ ] Design lazy surface creation (1h)
- [ ] Defer MPV init (3h)
- [ ] Add visibility detection (2h)
- [ ] DPMS state awareness (2h)
- [ ] Test hotplug scenarios (2h)

### Frame Skip Intelligence
- [ ] Implement skip heuristics (3h)
- [ ] Add GPU load detection (2h)
- [ ] Dynamic FPS adjustment (3h)
- [ ] Add config options (1h)
- [ ] Test under load (2h)

**Total P1 Estimate**: 50 hours (~7 days)

---

## 🚀 Phase 2: Features (Week 2) - P1

### HDR Support
- [ ] Detect HDR content (2h)
- [ ] Query output HDR caps (3h)
- [ ] Configure MPV HDR (2h)
- [ ] Add tone mapping (3h)
- [ ] Add config option (1h)
- [ ] Test with HDR videos (3h)

### Advanced Multi-Monitor
- [ ] Per-output source override (3h)
- [ ] Different WE projects (2h)
- [ ] Output name patterns (2h)
- [ ] Priority/fallback logic (2h)
- [ ] Add ctl commands (2h)
- [ ] Test scenarios (2h)

### Playlist Support
- [ ] Support Directory source (3h)
- [ ] Add rotation logic (2h)
- [ ] Implement transitions (3h)
- [ ] Add ctl next/prev (2h)
- [ ] Support M3U/PLS (2h)
- [ ] Test and document (2h)

**Total P2 Estimate**: 45 hours (~6 days)

---

## 💎 Phase 3: Polish (Week 2-3) - P2

### Error Handling
- [ ] Desktop notifications (3h)
- [ ] Error recovery strategies (4h)
- [ ] Health check command (2h)
- [ ] Graceful degradation (3h)
- [ ] Error codes & docs (2h)

### Config Validator
- [ ] check-config command (2h)
- [ ] Hot-reload validation (2h)
- [ ] File checks (1h)
- [ ] Capability checks (2h)
- [ ] Suggestion engine (2h)

### Setup Wizard
- [ ] Interactive command (4h)
- [ ] Compositor detection (2h)
- [ ] Hardware detection (2h)
- [ ] Config generation (3h)
- [ ] Autostart setup (2h)

### Diagnostic Tools
- [ ] stats command (3h)
- [ ] Performance overlay (3h)
- [ ] Frame histogram (2h)
- [ ] Property logging (1h)
- [ ] JSON export (2h)

**Total P3 Estimate**: 45 hours (~6 days)

---

## 📦 Phase 4: Distribution (Week 3) - P2

### Debian/Ubuntu
- [ ] Create debian/ structure (3h)
- [ ] Write control files (2h)
- [ ] Add to PPA (2h)
- [ ] Test on versions (3h)
- [ ] Documentation (1h)

### Fedora/RPM
- [ ] Create spec file (2h)
- [ ] Submit to COPR (2h)
- [ ] Test on Fedora (2h)
- [ ] Documentation (1h)

### Flatpak
- [ ] Create manifest (3h)
- [ ] Handle permissions (2h)
- [ ] Test sandbox (2h)
- [ ] Submit to Flathub (2h)

### ARM64 Support
- [ ] Add aarch64 CI (2h)
- [ ] Setup cross-compile (3h)
- [ ] Test on RPi (3h)
- [ ] Create ARM AppImage (2h)

**Total P4 Estimate**: 37 hours (~5 days)

---

## 🧪 Testing & Documentation

### Testing
- [ ] Performance tests (4h)
- [ ] Integration tests (6h)
- [ ] Compatibility tests (4h)
- [ ] Stress tests (3h)

### Documentation
- [ ] Performance guide (3h)
- [ ] HDR guide (2h)
- [ ] Troubleshooting (3h)
- [ ] Lua API reference (4h)
- [ ] Install guides (3h)
- [ ] Update README (2h)
- [ ] Update DEV_NOTES (1h)

**Total Testing/Docs**: 35 hours (~4 days)

---

## 📊 Summary

| Phase | Priority | Estimated Hours | Days |
|-------|----------|----------------|------|
| P1: Performance | P0 | 50 | 7 |
| P2: Features | P1 | 45 | 6 |
| P3: Polish | P2 | 45 | 6 |
| P4: Distribution | P2 | 37 | 5 |
| Testing/Docs | - | 35 | 4 |
| **Total** | | **212** | **~28** |

**Realistic Timeline**: 3-4 weeks (accounting for buffer)

---

## 🏁 Sprint Planning

### Sprint 1 (Days 1-7): Performance Foundation
**Goal**: Implement all P0 features, validate performance improvements

**Daily Breakdown**:
- Day 1-2: Shared decode architecture + implementation
- Day 3-4: Memory optimization
- Day 5-6: Lazy init + frame skip
- Day 7: Testing + benchmarking

**Deliverables**:
- Shared decode working with 4 outputs
- Memory usage reduced by 30%+
- All tests passing

### Sprint 2 (Days 8-14): Advanced Features
**Goal**: Implement HDR, multi-monitor, playlist

**Daily Breakdown**:
- Day 8-9: HDR support
- Day 10-11: Multi-monitor enhancements
- Day 12-13: Playlist support
- Day 14: Integration testing

**Deliverables**:
- HDR videos play correctly
- Per-output configs working
- Playlist rotation functional

### Sprint 3 (Days 15-21): Polish & UX
**Goal**: Error handling, validation, diagnostics

**Daily Breakdown**:
- Day 15-16: Error handling + notifications
- Day 17-18: Config validator + setup wizard
- Day 19-20: Diagnostic tools
- Day 21: Testing + refinement

**Deliverables**:
- Better error messages
- Interactive setup working
- Stats command functional

### Sprint 4 (Days 22-28): Distribution & Release
**Goal**: Package for all platforms, prepare release

**Daily Breakdown**:
- Day 22-23: Debian/Ubuntu packages
- Day 24-25: Fedora/Flatpak
- Day 26: ARM64 support
- Day 27-28: Final testing, documentation, release prep

**Deliverables**:
- Packages on 3+ repos
- Documentation complete
- v0.4.0 ready

---

## 🎯 Daily Checklist Template

```markdown
### Day X - [Feature Name]

**Goals**:
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3

**Blockers**: None / [describe]

**Notes**: [Any important observations]

**Tomorrow**: [Next tasks]
```

---

## 🔄 Progress Tracking

Update this section daily:

### Week 1: Performance
- [ ] Mon: 
- [ ] Tue: 
- [ ] Wed: 
- [ ] Thu: 
- [ ] Fri: 
- [ ] Sat: 
- [ ] Sun: 

### Week 2: Features
- [ ] Mon: 
- [ ] Tue: 
- [ ] Wed: 
- [ ] Thu: 
- [ ] Fri: 
- [ ] Sat: 
- [ ] Sun: 

### Week 3: Polish
- [ ] Mon: 
- [ ] Tue: 
- [ ] Wed: 
- [ ] Thu: 
- [ ] Fri: 
- [ ] Sat: 
- [ ] Sun: 

### Week 4: Distribution
- [ ] Mon: 
- [ ] Tue: 
- [ ] Wed: 
- [ ] Thu: 
- [ ] Fri: 
- [ ] Sat: 
- [ ] Sun: 

---

**Status**: 📋 Not Started  
**Start Date**: TBD  
**Target End**: TBD  
**Actual End**: TBD
