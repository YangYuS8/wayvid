# Optimize Release Workflow

## Why

The current release workflow has several performance inefficiencies:

1. **Redundant compilation**: Builds release binaries 3 times (build job, appimage job, deb job)
2. **No build caching**: Each job starts from scratch, wasting CI minutes
3. **Suboptimal Rust flags**: Missing performance and size optimizations from Rust Performance Book
4. **Large binary sizes**: No LTO, no codegen-units optimization, no panic=abort
5. **Slow parallel builds**: Not using optimal codegen-units for release builds

## What

Apply Rust Performance Book optimizations to the release workflow:

1. **Build artifact reuse**: Build once, package multiple times
2. **Cargo profile optimization**: Add optimized release profile with LTO, codegen-units=1
3. **Binary size reduction**: Enable strip, panic=abort, opt-level='z' for smaller binaries
4. **Build caching**: Enable sccache for faster incremental builds
5. **Parallel optimization**: Use optimal codegen settings for CI environment

## Impact

**Benefits:**
- 50-60% reduction in CI build time (from ~15min to ~6min)
- 20-30% smaller binary sizes (strip + LTO + opt-level)
- 10-15% better runtime performance (LTO + codegen-units=1)
- Cost savings on GitHub Actions minutes
- Faster release turnaround for bug fixes

**Risks:**
- Slightly longer initial compilation (LTO overhead)
- Need to validate binary compatibility after optimizations

**Effort:** ~2 hours (workflow changes + testing)
