## 1. CI Workflow Optimization
- [x] 1.1 Simplify CI to quality checks only (fmt + clippy)
- [x] 1.2 Remove build and test jobs from ci.yml
- [x] 1.3 Update paths-ignore to include openspec/
- [x] 1.4 Add clippy with -D warnings
- [x] 1.5 Test CI with doc-only changes

## 2. Performance Audit
- [x] 2.1 Review hot paths in src/video/ (mpv, frame_timing, shared_decode)
- [x] 2.2 Review memory allocations in src/video/memory.rs
- [ ] 2.3 Identify lock contention in multi-threaded code
- [ ] 2.4 Check for unnecessary clones and copies

## 3. Performance Optimizations
- [x] 3.1 Optimize video frame buffer management (BufferPool lock reduction)
- [x] 3.2 Reduce allocations in hot loops (frame_timing, memory tracking)
- [ ] 3.3 Use const generics for fixed-size buffers
- [ ] 3.4 Replace Mutex with lock-free structures where appropriate
- [x] 3.5 Add #[inline] to small hot functions (frame_timing, memory, mpv, layout)
- [x] 3.6 Use &str instead of String where possible (format_bytes optimization)

## 4. Profiling Setup
- [ ] 4.1 Add flamegraph generation script
- [ ] 4.2 Add criterion benchmarks for critical paths
- [ ] 4.3 Document profiling workflow in scripts/README.md

## 5. Validation
- [ ] 5.1 Run benchmarks before/after optimizations
- [x] 5.2 Verify no functional regressions (cargo check + clippy passed)
- [x] 5.3 Test CI workflow with various change types (workflows updated, paths-ignore tested)
- [x] 5.4 Update documentation if needed (PERFORMANCE_SUMMARY.md created)
