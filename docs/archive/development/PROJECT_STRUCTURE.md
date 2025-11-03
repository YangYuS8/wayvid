# wayvid - Project Structure

## Directory Layout

```
wayvid/
├── .github/
│   └── workflows/
│       └── ci.yml                    # GitHub Actions CI pipeline
│
├── configs/
│   ├── config.example.yaml           # Example configuration with all options
│   ├── hyprland-autostart.conf       # Hyprland integration example
│   ├── niri-autostart.kdl            # niri integration example
│   └── we-import.example.yaml        # Wallpaper Engine import example (future)
│
├── packaging/
│   ├── aur/
│   │   └── PKGBUILD                  # Arch Linux AUR package
│   ├── appimage/                     # AppImage build scripts (M2)
│   ├── nix/                          # Moved to root as flake.nix
│   ├── flatpak/                      # Flatpak manifest (M3)
│   ├── deb/                          # Debian package scripts (M3)
│   └── rpm/                          # RPM package scripts (M3)
│
├── scripts/
│   └── dev-check.sh                  # Development environment checker
│
├── src/
│   ├── main.rs                       # Entry point, CLI argument parsing
│   ├── config.rs                     # Configuration loading and management
│   │
│   ├── core/
│   │   ├── mod.rs
│   │   ├── layout.rs                 # Layout calculation (Fill/Contain/etc.)
│   │   └── types.rs                  # Shared types (VideoSource, LayoutMode, etc.)
│   │
│   ├── backend/
│   │   ├── mod.rs
│   │   └── wayland/
│   │       ├── mod.rs
│   │       ├── app.rs                # Main event loop and state management
│   │       ├── output.rs             # Output (monitor) tracking
│   │       └── surface.rs            # Layer surface wrapper
│   │
│   ├── video/
│   │   ├── mod.rs
│   │   ├── mpv.rs                    # libmpv integration (MVP version)
│   │   └── gst.rs                    # GStreamer backend (future, M4+)
│   │
│   └── ctl/
│       ├── mod.rs
│       ├── check.rs                  # System capability checker
│       └── ipc.rs                    # IPC interface (future, M4)
│
├── systemd/
│   └── wayvid.service                # systemd user service unit
│
├── .gitignore
├── Cargo.toml                        # Rust project manifest
├── Cargo.lock                        # Dependency lock file (generated)
├── flake.nix                         # Nix flake for NixOS users
├── LICENSE-MIT                       # MIT license
├── LICENSE-APACHE                    # Apache 2.0 license (future)
├── README.md                         # Main documentation
├── CONTRIBUTING.md                   # Contribution guidelines
├── QUICKSTART.md                     # Quick start guide
└── AI_PROMPT.md                      # Original development prompt
```

## Module Overview

### Core Modules

#### `src/main.rs`
- CLI argument parsing with `clap`
- Logging initialization with `tracing`
- Command dispatch (run, check, reload)

#### `src/config.rs`
- YAML/TOML configuration parsing
- Per-output configuration overrides
- Effective configuration calculation

#### `src/core/`
- **layout.rs**: Video-to-output layout calculation
  - Fill: Scale and crop to fill
  - Contain: Scale to fit with letterbox
  - Stretch: Ignore aspect ratio
  - Centre: No scaling
- **types.rs**: Shared data structures
  - `VideoSource`: File/Directory/WeProject
  - `LayoutMode`: Layout modes enum
  - `OutputInfo`: Monitor information
  - `PlaybackState`, `HwdecMode`: Playback state

### Backend Module

#### `src/backend/wayland/`
- **app.rs**: Main Wayland event loop
  - Registry handling
  - Global discovery (compositor, layer-shell, outputs)
  - Event dispatching
  - Surface lifecycle management
  
- **output.rs**: Output (monitor) tracking
  - Geometry, mode, scale events
  - Position tracking
  
- **surface.rs**: Layer surface wrapper
  - wlr-layer-shell surface creation
  - Input passthrough configuration
  - EGL context setup (placeholder in MVP)
  - Frame rendering coordination

### Video Module

#### `src/video/mpv.rs`
- libmpv initialization
- Playback configuration (loop, speed, start time)
- Hardware decode setup
- Audio control
- Frame rendering (simplified in MVP)

### Control Module

#### `src/ctl/check.rs`
- Wayland connection check
- Protocol availability check
- Video backend verification
- OpenGL/EGL library check
- Hardware decode capability check
- Recommendations output

## Key Dependencies

### Runtime
- **wayland-client**: Wayland protocol client
- **wayland-protocols**: Standard Wayland protocols
- **wayland-protocols-wlr**: wlroots protocols (layer-shell)
- **smithay-client-toolkit**: High-level Wayland toolkit
- **libmpv**: Media playback via MPV
- **khronos-egl**: EGL bindings for OpenGL
- **gl**: OpenGL bindings

### Configuration & CLI
- **serde**: Serialization framework
- **serde_yaml**: YAML support
- **clap**: Command-line parsing
- **shellexpand**: Path expansion (~, $HOME)

### Error Handling & Logging
- **anyhow**: Error handling
- **thiserror**: Custom error types
- **tracing**: Structured logging
- **tracing-subscriber**: Log output

### Event Loop
- **calloop**: Event loop (via sctk)

## Feature Flags

```toml
[features]
default = ["video-mpv", "backend-wayland"]

video-mpv = ["dep:libmpv"]          # libmpv video backend (default)
video-gst = []                       # GStreamer backend (future)
backend-wayland = []                 # Wayland support (default)
config-toml = ["dep:toml"]          # TOML config format
ipc = []                             # IPC control interface (future)
telemetry = []                       # Performance metrics (future)
tray = []                            # System tray icon (future)
```

## Build Profiles

### Development (`cargo build`)
- Optimization level: 1 (some optimization for faster iteration)
- Debug symbols: Yes
- LTO: No

### Release (`cargo build --release`)
- Optimization level: 3 (maximum)
- Debug symbols: No (stripped)
- LTO: Thin
- Codegen units: 1 (better optimization)

## Testing Structure

### Unit Tests
- Located in each module with `#[cfg(test)]`
- Run with `cargo test`
- Examples: layout calculation, config parsing

### Integration Tests
- Require Wayland compositor
- Marked with `#[ignore]` for CI
- Run with `cargo test -- --ignored`

## CI/CD Pipeline

### GitHub Actions Workflow
1. **Check**: `cargo check --all-features`
2. **Test**: `cargo test --all-features`
3. **Clippy**: `cargo clippy -- -D warnings`
4. **Format**: `cargo fmt --check`
5. **Build**: Multi-target release builds
   - x86_64-unknown-linux-gnu
   - aarch64-unknown-linux-gnu (cross-compile)

## Configuration Flow

1. User creates `~/.config/wayvid/config.yaml`
2. `wayvid run` loads global config
3. For each output:
   - Apply per-output overrides if present
   - Create EffectiveConfig
   - Initialize player with effective config
4. Runtime: Config can be reloaded via IPC (future)

## Event Flow

1. **Initialization**
   - Connect to Wayland
   - Discover globals (compositor, layer-shell, outputs)
   - Create surfaces for each output
   
2. **Configuration**
   - Layer surface receives configure event
   - Set size to output dimensions
   - Acknowledge configuration
   - Initialize video player
   
3. **Rendering Loop** (simplified in MVP)
   - Frame callback from surface
   - Render video frame
   - Commit surface
   - Request next frame

4. **Hotplug** (M2+)
   - New output appears → create surface
   - Output removed → destroy surface
   - Output properties change → reconfigure

## Performance Considerations

### Current MVP
- No frame synchronization yet
- Simplified rendering (no actual GL)
- Per-output player instances

### M2 Optimizations
- Proper vsync via frame callbacks
- EGL/GL rendering pipeline
- Shared decode for same video (optional)

### M3+ Optimizations
- Zero-copy where possible
- DMA-BUF for direct rendering
- Adaptive quality based on power state

## Known Limitations (MVP)

1. **OpenGL Integration**: Placeholder only
   - Need full mpv_render_context
   - Need FBO → layer surface pipeline
   
2. **Single Output**: Multi-output works but not optimized
   - Each output has separate player
   - No hotplug detection yet
   
3. **Frame Timing**: No vsync
   - Renders as fast as possible
   - Will be fixed in M2

4. **Power Management**: Partial
   - Configuration present
   - Not fully implemented

## Development Roadmap Implementation

### M1 Deliverables ✅
- [x] Project structure
- [x] Wayland layer-shell integration
- [x] libmpv integration (simplified)
- [x] Configuration system
- [x] Layout calculation
- [x] CLI and capability check
- [x] Build system and packaging scaffolds
- [x] Documentation

### M2 Focus
- [ ] Full OpenGL/EGL rendering
- [ ] Frame callbacks and vsync
- [ ] Multi-output hotplug
- [ ] Power management
- [ ] Performance metrics

### M3 Focus
- [ ] Wallpaper Engine importer
- [ ] Distribution packages
- [ ] User documentation
- [ ] Troubleshooting guide

### M4 Focus
- [ ] IPC control
- [ ] Static image fallback
- [ ] Advanced features
- [ ] Performance optimization

---

This structure supports iterative development while maintaining clean separation of concerns. Each module has a clear responsibility and can be tested independently where possible.
