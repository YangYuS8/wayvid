## 1. Cargo Profile Optimization
- [x] 1.1 Add optimized release profile to Cargo.toml
- [x] 1.2 Enable LTO (Link Time Optimization) - thin LTO
- [x] 1.3 Set codegen-units=1 for maximum optimization
- [x] 1.4 Enable strip and panic=abort
- [x] 1.5 Add release-opt profile for fat LTO option

## 2. Workflow Build Optimization
- [x] 2.1 Add RUSTFLAGS for target-cpu and lld linker
- [x] 2.2 Enable build caching for faster builds
- [x] 2.3 Install lld and clang for optimized linking
- [x] 2.4 Set CARGO_INCREMENTAL=0 for better LTO
- [x] 2.5 Add binary size reporting
- [ ] 2.6 Consider sccache for cross-job caching (future)

## 3. Binary Packaging
- [x] 3.1 Appimage job already reuses build artifacts (NO_BUILD=1)
- [x] 3.2 Deb job already reuses build artifacts (cargo deb --no-build)
- [x] 3.3 Checksum verification already in place (SHA256SUMS)

## 4. Validation
- [x] 4.1 Test release build locally with new profile (cargo check passed)
- [x] 4.2 Create and push v0.4.4-alpha.1 tag
- [ ] 4.3 Monitor GitHub Actions release workflow
- [ ] 4.4 Verify binary sizes are reduced in artifacts
- [ ] 4.5 Test downloaded binaries functionality
- [ ] 4.6 Run performance benchmarks on optimized binaries (after release)
