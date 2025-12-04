# Change: Improve CI Efficiency and Performance Optimization

## Why
1. **CI Inefficiency**: Current CI workflow compiles and tests code even when only non-code files (docs, configs) are modified, wasting resources
2. **No Clippy Enforcement**: No systematic linting to catch code quality issues early
3. **Performance Gaps**: Codebase lacks systematic performance optimizations following Rust best practices

## What Changes
- **CI Workflow Refactor**: Transform CI to focus on quick quality checks (fmt + clippy) instead of full build
- **Smart Path Filtering**: Skip CI entirely when only docs/configs/packaging files are modified
- **Clippy Integration**: Add comprehensive clippy checks with deny-warnings
- **Performance Optimization**: Apply Rust Performance Book techniques to critical paths
  - Video decoding pipeline optimization
  - Memory allocation improvements
  - Lock-free data structures where possible
  - Profile-guided optimization setup

## Impact
- Affected workflows: `.github/workflows/ci.yml`
- Affected code: `src/video/`, `src/core/`, `src/backend/`
- Benefits:
  - Faster CI feedback (minutes → seconds for non-code changes)
  - Earlier detection of code quality issues
  - Improved runtime performance (estimated 10-30% reduction in CPU/memory)
  - Lower power consumption on battery

## Non-Goals
- Complete rewrite of video pipeline (incremental improvements only)
- Micro-optimizations without profiling data
- Breaking API changes
