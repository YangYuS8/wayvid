# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-01-XX

### Added
- **Desktop GUI (`wayvid-gui`)**: Visual control panel for managing wayvid daemon
  - Real-time status display (playback state, FPS, video info)
  - Pause/Resume controls with visual feedback
  - Connection status indicator (daemon running/stopped)
  - IPC-based communication with daemon
  - Enabled with `--features gui` (using egui/eframe)
- **Enhanced Diagnostic Tool (`wayvid-ctl check`)**: Comprehensive system capability checker
  - Wayland environment validation
  - Compositor compatibility detection (Hyprland, Niri, Sway, etc.)
  - Niri integration status and optimization checks
  - Video backend verification (libmpv)
  - OpenGL/EGL library detection
  - Hardware decode support (VA-API, VDPAU)
  - Daemon status checking
  - Configuration file validation
  - Beautiful Unicode box formatting with emoji indicators
- **IPC Client Module (`ctl::ipc_client`)**: Reusable IPC communication library
  - Unix socket connection management
  - Command serialization and response handling
  - Daemon running status check
  - Used by both CLI and GUI tools

### Improved
- **Error Messages**: Dramatically improved user-facing error messages across the codebase
  - Backend missing: Clear compilation instructions
  - IPC connection failures: Step-by-step troubleshooting guide
  - Config file errors: Distinguish between missing file and YAML syntax errors
  - WE project detection: Show expected project structure
  - WE import failures: Detailed cause analysis and fix suggestions
  - All errors now include context and actionable recommendations
- **Documentation**:
  - Niri integration docs aligned with actual implementation
  - Removed unimplemented feature claims
  - Enhanced configuration examples with Niri-specific guidance
  - Improved autostart configuration examples

### Fixed
- **AUR Packaging**: Added GUI support to PKGBUILD and PKGBUILD.stable
  - Now builds with `--features gui`
  - Installs all three binaries: `wayvid`, `wayvid-ctl`, `wayvid-gui`
- **Documentation Accuracy**: Corrected Niri feature documentation
  - Removed claims about unimplemented scroll optimization
  - Removed references to non-existent quality tuning settings
  - Updated to accurately reflect workspace FPS throttling as the sole optimization

## [0.3.0] - 2025-10-23

### Added

#### M4 Phase 1-2: Wallpaper Engine Import
- **WE Project Parser**: Full support for parsing Wallpaper Engine `project.json` files
  - Type definitions with proper serde deserialization
  - Property extraction (rate, volume, playback mode, alignment, audio processing)
  - Video file path resolution
- **import Command**: CLI command to convert WE projects to wayvid config
  - Output to stdout or file
  - Metadata preservation (title, workshop ID, description)
  - Property mapping with automatic conversion
- **Property Mapping**:
  - Alignment: `0=Center, 1=Fit, 2=Fill, 3=Stretch` → `Centre/Contain/Cover/Fill`
  - Playback mode: `0=loop, 1=pause` → `loop: true/false`
  - Volume: `0-100` → `0.0-100.0`
  - Rate: Direct mapping
- **Documentation**: Complete WE format reference in `docs/WE_FORMAT.md`

#### M4 Phase 3: AUR Packaging
- **PKGBUILD**: Arch Linux package definition for `wayvid-git`
- **PKGBUILD.stable**: Stable release package for tagged versions
- **.SRCINFO**: AUR metadata for package database
- **Dependencies**:
  - Runtime: `wayland`, `libmpv`, `gcc-libs`, `glibc`
  - Optional: `mesa`, `libva-intel-driver`, `libva-mesa-driver`, `nvidia-utils`
- **test-package.sh**: Automated PKGBUILD validation and testing
- **Documentation**: Complete AUR packaging guide in `packaging/aur/README.md`

#### M4 Phase 4: Nix Flake
- **flake.nix**: Modern Nix package definition
  - Integration with `rust-overlay` for latest Rust toolchains
  - Complete package with systemd service and documentation
  - Multiple apps: `wayvid`, `wayvid-ctl`
  - Enhanced development shell with cargo-watch, cargo-edit, rust-analyzer
- **Installation Methods**:
  - Direct run: `nix run github:YangYuS8/wayvid`
  - Profile install: `nix profile install github:YangYuS8/wayvid`
  - NixOS module integration
  - Home Manager integration
- **Documentation**: Comprehensive Nix guide in `packaging/nix/README.md` (351 lines)

#### M4 Phase 5: AppImage Packaging
- **AppImage Build System**: Universal Linux binary distribution
  - `AppRun` script with mode detection (wayvid/wayvid-ctl)
  - Desktop entry and icon
  - Automated build script with dependency bundling
  - UPX compression support
  - Size and checksum reporting
- **Testing Infrastructure**: Comprehensive test script for validation
  - Basic functionality tests
  - Dependency checking
  - Content inspection
  - Multi-distribution compatibility
- **CI/CD**: GitHub Actions workflow for automated builds
  - Build on tags
  - Manual workflow dispatch
  - Artifact uploads
  - Release integration
- **Documentation**: Complete AppImage guide in `packaging/appimage/README.md`
- **Features**:
  - Self-contained with all dependencies
  - Works on any Linux distribution
  - No root required
  - Both wayvid and wayvid-ctl included

#### M4 Phase 6: Documentation Updates
- **README.md**: Major updates
  - Installation section rewrite with AppImage priority
  - AUR installation instructions
  - Nix Flakes installation guide
  - Wallpaper Engine import section
  - Updated status to M4 Complete
  - Distribution support summary
  - Systemd service setup
- **QUICKSTART.md**: Streamlined quick start
  - AppImage as primary installation method
  - WE import as configuration option A
  - Runtime control examples
  - Multiple installation paths
- **Cargo.toml**: Version bump to 0.3.0, repository URL update
- **CHANGELOG.md**: This file!

### Fixed

#### M4 Phase 1
- **serde untagged enum ordering**: Fixed Combo/Slider deserialization
  - Issue: Slider's `f64` matched Combo's `i64` values, causing wrong property extraction
  - Solution: Reordered enum variants (Combo before Slider)
  - Impact: Alignment property now correctly parses (e.g., `2` → `Cover` not `Contain`)

### Changed

- **Version**: 0.1.0 → 0.3.0 (reflecting M1-M4 completion)
- **Repository**: Updated from placeholder to `https://github.com/YangYuS8/wayvid`
- **Status**: M3 → M4 Complete

## [0.2.0] - 2025-10-XX (M3)

### Added
- Runtime control API via `wayvid-ctl`
- Configuration hot reload
- Multi-source support (Files, URLs, RTSP, Pipes, Images)
- Per-output configuration overrides

### Changed
- Enhanced power management
- Improved documentation structure

## [0.1.0] - 2025-10-XX (M1-M2)

### Added
- Initial Wayland layer-shell implementation
- OpenGL/EGL rendering with mpv
- Multi-output support with hotplug
- Hardware decode (VA-API/NVDEC)
- Layout modes (Fill/Contain/Stretch/Cover/Centre)
- Basic power management
- Configuration system
- Example configs and documentation

[0.3.0]: https://github.com/YangYuS8/wayvid/releases/tag/v0.3.0
[0.2.0]: https://github.com/YangYuS8/wayvid/releases/tag/v0.2.0
[0.1.0]: https://github.com/YangYuS8/wayvid/releases/tag/v0.1.0
