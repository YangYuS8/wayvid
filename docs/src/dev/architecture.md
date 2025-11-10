# Architecture

wayvid's high-level architecture and design decisions.

## Overview

```
┌─────────────────┐
│  Configuration  │
└────────┬────────┘
         │
    ┌────▼─────┐
    │   Core   │ ── Event Loop
    └────┬─────┘
         │
    ┌────▼──────────────────────┐
    │   Wayland Backend         │
    │  ┌──────┐  ┌──────────┐  │
    │  │Layer │  │  Output  │  │
    │  │Shell │  │  Manager │  │
    │  └──────┘  └──────────┘  │
    └────┬──────────────────────┘
         │
    ┌────▼─────────────────┐
    │   Video Subsystem    │
    │  ┌──────┐  ┌──────┐ │
    │  │ mpv  │  │ EGL  │ │
    │  └──────┘  └──────┘ │
    └──────────────────────┘
```

## Core Components

### Configuration (`src/config/`)
- YAML parsing
- Type validation
- Hot-reload support
- Per-output config

### Core (`src/core/`)
- Main event loop
- State management
- Output layout tracking
- Power management

### Wayland Backend (`src/backend/wayland/`)
- Layer-shell protocol
- Output management
- Surface creation
- Event handling

### Video (`src/video/`)
- mpv integration
- EGL rendering
- Hardware decode
- HDR pipeline

### IPC (`src/ctl/`)
- Unix socket server
- Protocol handling
- Command dispatch

## Data Flow

```
Config → Core → Backend → Video
                  ↓         ↓
              Wayland ← EGL Surface
```

1. Config parsed and validated
2. Core initializes subsystems
3. Backend creates Wayland surfaces
4. Video renders to EGL surfaces
5. IPC accepts runtime commands

## Key Design Decisions

### Layer-Shell
Uses `wlr-layer-shell` protocol for background placement:
- Guaranteed bottom layer
- No window management needed
- Works with all wlr compositors

### mpv Integration
Uses libmpv for video:
- Mature, stable
- Hardware acceleration
- HDR support
- Wide format support

### EGL Rendering
Direct EGL context sharing:
- Zero-copy textures
- GPU-accelerated
- Wayland-native

### Async Architecture
Tokio-based async runtime:
- Non-blocking I/O
- Efficient resource usage
- Clean concurrency

## Module Breakdown

```
src/
├── backend/        # Wayland integration
│   └── wayland/
│       ├── app.rs      # Main app state
│       ├── output.rs   # Output management
│       └── surface.rs  # Surface handling
│
├── config/         # Configuration
│   ├── types.rs    # Config structs
│   ├── watcher.rs  # File watching
│   └── pattern.rs  # Source patterns
│
├── core/           # Core logic
│   ├── layout.rs   # Output layout
│   ├── power.rs    # Power management
│   └── types.rs    # Core types
│
├── ctl/            # IPC control
│   ├── ipc_server.rs   # Unix socket server
│   ├── protocol.rs     # Command protocol
│   └── check.rs        # Health checks
│
├── video/          # Video subsystem
│   ├── mpv.rs          # mpv integration
│   ├── egl.rs          # EGL context
│   ├── hdr.rs          # HDR pipeline
│   └── frame_timing.rs # Frame pacing
│
└── we/             # Workshop support
    ├── parser.rs       # WE project parser
    ├── converter.rs    # WE → native
    ├── steam.rs        # Steam integration
    └── workshop.rs     # Workshop scanner
```

## Threading Model

- **Main Thread**: Wayland event loop
- **Video Thread**: mpv decoding (internal)
- **IPC Thread**: Command handling
- **Async Runtime**: I/O operations

## Memory Management

- Surfaces: Shared GPU memory
- Video frames: mpv-managed
- Config: Arc-wrapped for sharing
- IPC: Short-lived allocations

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|--------|
| Output add | O(1) | |
| Frame render | O(n outputs) | Parallelizable |
| Config reload | O(1) | Hot-reload |
| IPC command | O(1) | Async |

## Future Architecture

Planned improvements:
- Pluggable video backends
- Multiple video engines
- Shader system
- Advanced compositing
