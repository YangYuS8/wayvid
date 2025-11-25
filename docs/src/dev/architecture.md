# Architecture

wayvid's high-level architecture and design decisions.

## Overview

```
┌─────────────────┐     ┌─────────────────┐
│  Configuration  │     │   wayvid-gui    │
└────────┬────────┘     └────────┬────────┘
         │                       │ IPC
    ┌────▼─────┐          ┌──────▼──────┐
    │   Core   │◄─────────│ IPC Server  │
    └────┬─────┘          └─────────────┘
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
- YAML parsing with serde
- Type validation
- Hot-reload support via file watcher
- Per-output config with pattern matching

### Core (`src/core/`)
- Main event loop (non-blocking poll)
- State management
- Output layout tracking
- Power management (battery, hidden window)

### Wayland Backend (`src/backend/wayland/`)
- wlr-layer-shell protocol
- Output management (hotplug support)
- Surface creation with `exclusive_zone(-1)` for full coverage
- Event handling with libc poll

### Video (`src/video/`)
- mpv integration via libmpv
- EGL rendering with shared decode context
- Hardware decode acceleration
- HDR pipeline with tone mapping

### IPC (`src/ctl/`)
- Unix socket server with request-response pattern
- Protocol handling (JSON over newline-delimited stream)
- Bidirectional communication for status queries
- Command dispatch with response callbacks

### GUI (`src/bin/wayvid-gui.rs`)
- egui/eframe desktop application
- Internationalization (i18n) with rust-i18n
- System locale detection
- Async IPC client in separate thread

## Data Flow

```
Config → Core → Backend → Video
                  ↓         ↓
              Wayland ← EGL Surface

wayvid-ctl/GUI → IPC Socket → IPC Server → Core
                      ↑           ↓
                   Response ← Command Handler
```

1. Config parsed and validated
2. Core initializes subsystems
3. Backend creates Wayland surfaces
4. Video renders to EGL surfaces
5. IPC accepts runtime commands with responses

## IPC Architecture

The IPC system uses a **request-response** pattern:

```
┌────────────┐    Request     ┌────────────┐
│   Client   │───────────────>│   Server   │
│ (ctl/gui)  │<───────────────│  (daemon)  │
└────────────┘    Response    └────────────┘
```

### Request Flow
1. Client connects to Unix socket
2. Client sends JSON command (newline-terminated)
3. Server creates response channel per request
4. Command dispatched to main thread
5. Handler generates response
6. Response sent back through channel
7. Server writes response to client

### Non-blocking Event Loop
```rust
while running {
    // 1. Dispatch pending Wayland events
    event_queue.dispatch_pending(&mut state)?;
    
    // 2. Poll for new events (16ms timeout)
    poll(&mut [wayland_fd], 16)?;
    
    // 3. Process IPC requests (non-blocking)
    while let Ok(request) = request_rx.try_recv() {
        let response = handle_command(request.command);
        request.response_tx.send(response)?;
    }
    
    // 4. Handle config changes, power management, etc.
}
```

## Key Design Decisions

### Layer-Shell with Full Coverage
Uses `wlr-layer-shell` protocol with `exclusive_zone(-1)`:
- Guaranteed bottom layer
- Ignores panel/bar exclusive zones
- Works with all wlr compositors (Hyprland, Sway, Niri, etc.)

### mpv Integration
Uses libmpv for video:
- Mature, stable
- Hardware acceleration (VAAPI, VDPAU, NVDEC)
- HDR support with tone mapping
- Wide format support

### EGL Rendering
Direct EGL context sharing:
- Zero-copy textures
- GPU-accelerated
- Wayland-native
- Shared decode context for multi-monitor

### Internationalization
GUI supports multiple languages:
- rust-i18n with TOML locale files
- System locale auto-detection
- Runtime language switching
- Fallback to English

## Module Breakdown

```
src/
├── backend/        # Wayland integration
│   ├── wayland/
│   │   ├── app.rs      # Main app state & event loop
│   │   ├── output.rs   # Output management
│   │   └── surface.rs  # Surface handling
│   └── niri.rs     # Niri compositor integration
│
├── bin/            # Executable binaries
│   ├── wayvid-ctl.rs   # CLI control tool
│   └── wayvid-gui.rs   # Desktop GUI application
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
│   ├── ipc_client.rs   # Client library
│   ├── protocol.rs     # Command protocol
│   └── check.rs        # Health checks
│
├── video/          # Video subsystem
│   ├── mpv.rs          # mpv integration
│   ├── egl.rs          # EGL context
│   ├── hdr.rs          # HDR pipeline
│   ├── shared_decode.rs # Multi-monitor optimization
│   └── frame_timing.rs # Frame pacing
│
├── we/             # Workshop support
│   ├── parser.rs       # WE project parser
│   ├── converter.rs    # WE → native
│   ├── steam.rs        # Steam integration
│   └── workshop.rs     # Workshop scanner
│
└── locales/        # i18n translations
    ├── en.toml     # English
    └── zh-CN.toml  # Simplified Chinese
```

## Threading Model

- **Main Thread**: Wayland event loop + IPC dispatch
- **Video Thread**: mpv decoding (mpv-internal)
- **IPC Listener Thread**: Accept connections, parse commands
- **GUI IPC Thread**: Async communication with daemon

## Memory Management

- Surfaces: Shared GPU memory via EGL
- Video frames: mpv-managed with reference counting
- Config: Arc-wrapped for sharing across threads
- IPC: Short-lived allocations, channel-based messaging

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|--------|
| Output add | O(1) | Hotplug support |
| Frame render | O(n outputs) | Shared decode context |
| Config reload | O(1) | Hot-reload |
| IPC command | O(1) | Non-blocking |
| Status query | O(n outputs) | Gathers all output info |

## Future Architecture

Planned improvements:
- WebSocket IPC for remote control
- Plugin system for video backends
- Custom shader effects
- Scene/playlist management
