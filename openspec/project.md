# Project Context

## Purpose
wayvid is a lightweight, high-performance dynamic video wallpaper application for Wayland, designed as a native Linux alternative to Wallpaper Engine. It follows a **GUI-first** design philosophy where users can simply open the app, browse their wallpaper library, and apply wallpapers with a double-click.

**Core Goals:**
- **GUI-First**: Primary interaction through graphical interface, not command line
- **Wallpaper Engine Compatibility**: Import and play Steam Workshop content
- **Lightweight & High-Performance**: Minimal resource usage with hardware acceleration
- **Native Wayland**: Built specifically for modern Linux desktops
- **Zero Configuration**: Works out of the box, settings managed through GUI
- **Multi-Monitor**: Independent wallpapers per display

**Design Principles:**
- Users should never need to write config files
- Close window = minimize to tray (wallpaper keeps playing)
- Thumbnails load asynchronously, never block UI
- CLI is optional, for scripting/automation only

## Tech Stack

### Core Technologies
- **Language**: Rust 1.75+ (Edition 2021)
- **Graphics**: OpenGL ES 3.0, EGL, VA-API/NVDEC, wgpu
- **Wayland**: wlr-layer-shell, smithay-client-toolkit
- **Video**: libmpv (hardware decode backend)
- **GUI**: iced (Elm-style declarative UI, GPU-accelerated via wgpu)
- **Build**: Cargo with workspace structure

### Key Dependencies
- `wayland-client` 0.31 - Wayland protocol bindings
- `smithay-client-toolkit` 0.19 - High-level Wayland client
- `libmpv-sys` 3.1 - Video playback backend
- `khronos-egl` 6.0 - EGL context management
- `serde` + `serde_yaml` - Configuration parsing
- `clap` 4.5 - CLI argument parsing
- `tracing` + `tracing-subscriber` - Structured logging
- `calloop` 0.13 - Event loop
- `iced` 0.13 - Declarative GUI framework (wgpu backend)

### Supported Platforms
- **Primary**: Arch Linux, NixOS
- **Compositors**: Hyprland (full), Niri (full), Sway (partial), River (partial)
- **Architectures**: x86_64, aarch64

## Project Conventions

### Code Style
- **Formatting**: `rustfmt` with default settings
- **Linting**: `clippy` warnings must be resolved
- **Naming**:
  - Modules: `snake_case` (e.g., `video_player`, `config_loader`)
  - Types: `PascalCase` (e.g., `VideoSource`, `OutputConfig`)
  - Functions/methods: `snake_case` (e.g., `init_video`, `handle_event`)
  - Constants: `UPPER_SNAKE_CASE` (e.g., `DEFAULT_FPS`, `MAX_BUFFERS`)
- **Error Handling**: Use `anyhow::Result` for applications, `thiserror` for libraries
- **Logging**: `tracing` macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`)
- **Documentation**: Public APIs require doc comments with examples

### Architecture Patterns

#### Module Structure
```
src/
├── main.rs              # Daemon entry point
├── lib.rs               # Shared library code
├── backend/             # Wayland backend abstraction
│   ├── mod.rs           # Backend trait + common types
│   ├── wayland/         # wlr-layer-shell implementation
│   └── niri.rs          # Niri-specific optimizations
├── config/              # Configuration management
│   ├── types.rs         # Config data structures
│   ├── pattern.rs       # Output pattern matching
│   └── watcher.rs       # Hot-reload support
├── core/                # Core engine logic
│   ├── layout.rs        # Video scaling/positioning
│   ├── types.rs         # Core data types
│   └── power.rs         # Power management
├── video/               # Video playback layer
│   ├── mpv.rs           # libmpv integration
│   ├── egl.rs           # EGL/OpenGL rendering
│   ├── hdr.rs           # HDR pipeline
│   ├── frame_timing.rs  # Frame pacing
│   ├── memory.rs        # Buffer management
│   └── shared_decode.rs # Decoder sharing
├── ctl/                 # IPC control
│   ├── ipc_server.rs    # Unix socket server
│   ├── ipc_client.rs    # Client library
│   └── protocol.rs      # Command/response types
├── we/                  # Wallpaper Engine integration
│   ├── parser.rs        # project.json parser
│   ├── converter.rs     # WE → wayvid config
│   ├── workshop.rs      # Steam Workshop scanner
│   ├── downloader.rs    # Workshop downloader
│   └── steam.rs         # Steam library detection
└── bin/                 # Additional binaries
    ├── wayvid-ctl.rs    # CLI control tool
    └── wayvid-gui.rs    # GUI control panel
```

#### Design Principles
- **Separation of Concerns**: Backend, core logic, video, and config are decoupled
- **Trait-Based Abstractions**: `Backend` trait allows compositor-specific implementations
- **Single Responsibility**: Each module has one clear purpose
- **Fail-Fast**: Invalid configs/states should error immediately during startup
- **Zero-Copy**: Use shared memory and DMA-BUF for video frames
- **Event-Driven**: `calloop` for async I/O, Wayland events, and timers

#### Key Abstractions
- `Backend` trait: Abstracts Wayland compositor differences
- `VideoSource`: Unified interface for file, directory, workshop sources
- `OutputConfig`: Per-monitor configuration with pattern matching
- `SharedDecoder`: Shares decoded frames across multiple outputs
- `FrameTiming`: Handles frame pacing and FPS throttling

### Testing Strategy

#### Test Coverage
- **Unit Tests**: All core algorithms (`layout.rs`, `pattern.rs`, `power.rs`)
- **Integration Tests**: Config parsing, IPC protocol, WE conversion
- **Mock Tests**: Wayland interactions use mocks (no display server needed)
- **Property Tests**: Pattern matching, scaling algorithms (when applicable)

#### Running Tests
```bash
cargo test                    # Run all tests
cargo test --all-features     # Include optional features
cargo test -- --nocapture     # Show println! output
```

#### Test Organization
- Unit tests: `#[cfg(test)] mod tests` in same file
- Integration tests: `tests/` directory (if needed)
- Benchmarks: `benches/` directory (criterion)

#### CI Requirements
- All tests must pass on `main` branch
- Code coverage should not decrease (tracked via codecov)
- Ignored tests: `#[ignore]` for network/hardware-dependent tests

### Git Workflow

#### Branch Strategy
- **main**: Stable releases, protected branch
- **hotfix branches**: `hotfix/v0.4.x` for urgent fixes
- **feature branches**: Short-lived, merged via PR (if collaborating)

#### Commit Conventions (Conventional Commits)
Format: `<type>(<scope>): <subject>`

**⚠️ IMPORTANT: All Git messages (commits, branches, tags, PR titles) MUST be written in English.**

This applies to:
- Commit subjects and bodies
- Branch names
- Tag names and annotations
- Pull request titles and descriptions

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code restructuring (no behavior change)
- `perf`: Performance improvement
- `test`: Add/update tests
- `chore`: Tooling, dependencies, configs
- `ci`: CI/CD changes

**Scopes:**
- `config`: Configuration system
- `video`: Video playback
- `backend`: Wayland backend
- `gui`: GUI control panel
- `ctl`: CLI/IPC
- `we`: Wallpaper Engine integration
- `aur`: AUR packaging
- `nix`: Nix packaging

**Examples:**
```
feat(we): Add Steam Workshop downloader
fix(video): Fix memory leak in shared decoder
docs: Update installation instructions
refactor(backend): Simplify output hotplug logic
perf(video): Reduce frame copy overhead
chore(deps): Update wayland-protocols to 0.32
```

#### Release Process
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with release notes
3. Run `cargo update -p wayvid` to sync `Cargo.lock`
4. Commit: `chore: Release v0.x.y`
5. Tag: `git tag -a v0.x.y -m "Release v0.x.y"`
6. Push: `git push && git push --tags`
7. GitHub Actions automatically builds and publishes artifacts

## Domain Context

### Wayland Concepts
- **Layer Shell**: Protocol for overlays/backgrounds (wlr-layer-shell-unstable-v1)
- **Output**: Physical display (monitor), can be hotplugged
- **Surface**: Drawing target associated with an output
- **Subsurface**: Child surface for composition
- **Fractional Scaling**: HiDPI support via wp_fractional_scale_v1
- **Damage Tracking**: Only redraw changed regions

### Video Rendering Pipeline
1. **Decode**: libmpv decodes video (CPU or hardware VA-API/NVDEC)
2. **Upload**: Frames uploaded to GPU texture (EGL/OpenGL)
3. **Transform**: Apply scaling, HDR tone-mapping, color space conversion
4. **Present**: Rendered to Wayland surface via `wl_surface.commit`

### HDR Pipeline
- Input: 10-bit HDR (PQ, HLG) or SDR (BT.709)
- Processing: OpenGL shader for tone-mapping (ACES, Hable, Reinhard)
- Output: Matches display capabilities (HDR passthrough or SDR conversion)

### Multi-Monitor Semantics
- Each output has independent video source
- Pattern-based config: match outputs by name/model/serial
- Decoder sharing: Same video file decoded once, frames shared
- Hotplug: Dynamically add/remove outputs without restart

### Power Management
- **Battery Detection**: `/sys/class/power_supply/` monitoring
- **Workspace Awareness**: Pause when no workspace visible (Niri only)
- **FPS Throttling**: Reduce FPS on battery or high load
- **Idle Detection**: Pause after inactivity (compositor-dependent)

## Important Constraints

### Technical Constraints
- **Rust Version**: 1.75+ required (MSRV policy: update only for critical features)
- **Wayland Only**: No X11 support (by design)
- **GPU Required**: Software rendering not supported
- **libmpv Dependency**: Must be installed system-wide (dynamic linking)
- **EGL 1.5**: Required for DMA-BUF import
- **Compositor Compatibility**: Requires wlr-layer-shell protocol

### Performance Constraints
- **Memory**: Shared decoder reduces memory by 60% in multi-monitor setups
- **CPU**: Hardware decode required for 4K+ video (software decode too slow)
- **Frame Timing**: Must maintain vsync to avoid tearing
- **Startup Time**: < 2 seconds from launch to first frame

### Packaging Constraints
- **AUR**: Builds from source (git) or binary (stable)
- **Nix**: Flake-based, hermetic builds
- **Debian**: `cargo-deb` generated packages
- **AppImage**: Bundles all dependencies except libmpv/wayland

### Security Constraints
- **No Root**: Must run as unprivileged user
- **Config Files**: Only read from `~/.config/wayvid/` and XDG dirs
- **IPC Socket**: Unix domain socket with filesystem permissions
- **No Network**: Except Steam Workshop downloads (opt-in feature)

### Documentation Constraints
- **Primary Documentation**: Use `docs/` mdbook structure ONLY
- **No OpenSpec Documentation**: Do NOT create README.md, COVERAGE.md, or other markdown files in `openspec/`
- **OpenSpec Purpose**: Specifications only - no standalone documentation
- **Existing Docs**: Full documentation already exists in `docs/` as mdbook
- **Rule**: Never create additional documentation files without explicit request

## External Dependencies

### System Libraries (Runtime)
- `libmpv.so.2` - Video decoding (mpv)
- `libwayland-client.so` - Wayland protocol
- `libEGL.so.1` - EGL context
- `libGL.so.1` - OpenGL ES 3.0
- `libva.so.2` - VA-API (optional, hardware decode)
- `libnvidia-encode.so` - NVDEC (optional, NVIDIA only)

### Build Dependencies
- `rust` 1.75+ - Rust toolchain
- `cargo` - Build system
- `wayland-protocols` - Protocol definitions
- `mesa` - OpenGL headers
- `libxkbcommon` - Keyboard handling (GUI)
- `fontconfig` - Font rendering (GUI)

### Optional Runtime Dependencies
- `steam` - For Workshop integration
- `niri` - For workspace-aware optimizations
- `systemd` - For user service management

### External Services
- **Steam Workshop API**: Download Wallpaper Engine content
  - Endpoint: `https://api.steampowered.com/`
  - Rate limits: Standard Steam Web API limits
  - Authentication: Public API (no key required for read-only)

### Development Tools
- `clippy` - Linting
- `rustfmt` - Code formatting
- `mdbook` - Documentation generation
- `git-cliff` - Changelog generation
- `cargo-deb` - Debian package builder
- `appimagetool` - AppImage builder
