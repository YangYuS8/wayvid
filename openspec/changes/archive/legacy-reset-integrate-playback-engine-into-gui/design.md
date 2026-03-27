# Design: Integrated Playback Engine Architecture

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     wayvid-gui Process                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────┐    ┌───────────────────────────┐ │
│  │      iced UI          │    │    PlaybackEngine         │ │
│  │    (Main Thread)      │◄──►│   (Engine Thread)         │ │
│  │                       │    │                           │ │
│  │  ┌─────────────────┐  │    │  ┌─────────────────────┐  │ │
│  │  │ Library View    │  │    │  │ Wayland Connection  │  │ │
│  │  │ Monitors View   │  │    │  │ (wl_display)        │  │ │
│  │  │ Settings View   │  │    │  └─────────────────────┘  │ │
│  │  └─────────────────┘  │    │            │              │ │
│  │          │            │    │            ▼              │ │
│  │          │            │    │  ┌─────────────────────┐  │ │
│  │          ▼            │    │  │ OutputManager       │  │ │
│  │  ┌─────────────────┐  │    │  │ (track outputs)     │  │ │
│  │  │ Engine Control  │──┼────┼─►└─────────────────────┘  │ │
│  │  │ - Start/Stop    │  │    │            │              │ │
│  │  │ - Apply WP      │  │    │            ▼              │ │
│  │  │ - Clear WP      │  │    │  ┌─────────────────────┐  │ │
│  │  └─────────────────┘  │    │  │ WallpaperSession[]  │  │ │
│  │                       │    │  │ per-output:         │  │ │
│  └───────────────────────┘    │  │  - LayerSurface     │  │ │
│                               │  │  - EglContext       │  │ │
│  ┌───────────────────────┐    │  │  - MpvPlayer        │  │ │
│  │    IpcServer          │    │  └─────────────────────┘  │ │
│  │  (Optional, for ctl)  │◄───┤                           │ │
│  └───────────────────────┘    └───────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
        │                                    │
        ▼                                    ▼
┌───────────────────┐              ┌───────────────────┐
│   wayvid-ctl     │              │   Wayland         │
│  (CLI client)    │              │   Compositor      │
└───────────────────┘              └───────────────────┘
```

## Thread Model

### Main Thread (iced event loop)
- Handles all UI rendering and user interactions
- Processes iced messages
- Sends commands to engine via channel
- Receives status updates from engine

### Engine Thread (Wayland event loop)
- Manages Wayland connection and surfaces
- Runs MPV render callbacks
- Processes frame timing
- Reports status back to GUI

### Communication Channels

```rust
// GUI -> Engine commands
enum EngineCommand {
    Start,
    Stop,
    ApplyWallpaper { path: PathBuf, output: Option<String> },
    ClearWallpaper { output: Option<String> },
    SetVolume { output: String, volume: f32 },
    Pause { output: Option<String> },
    Resume { output: Option<String> },
}

// Engine -> GUI events
enum EngineEvent {
    Started,
    Stopped,
    OutputAdded(OutputInfo),
    OutputRemoved(String),
    WallpaperApplied { output: String, path: PathBuf },
    WallpaperCleared { output: String },
    Error(String),
}
```

## PlaybackEngine API

```rust
pub struct PlaybackEngine {
    /// Wayland connection (separate from iced's connection)
    connection: Connection,
    /// Output manager
    outputs: OutputManager,
    /// Active wallpaper sessions per output
    sessions: HashMap<String, WallpaperSession>,
    /// Command receiver
    commands: mpsc::Receiver<EngineCommand>,
    /// Event sender
    events: mpsc::Sender<EngineEvent>,
    /// Engine configuration
    config: EngineConfig,
}

impl PlaybackEngine {
    /// Create new engine instance
    pub fn new(
        config: EngineConfig,
        commands: mpsc::Receiver<EngineCommand>,
        events: mpsc::Sender<EngineEvent>,
    ) -> Result<Self>;

    /// Run engine event loop (blocking, call from dedicated thread)
    pub fn run(&mut self) -> Result<()>;

    /// Request shutdown (called from GUI thread via channel)
    pub fn request_shutdown(&self);
}
```

## WallpaperSession Lifecycle

```
                    ┌─────────────┐
                    │   Created   │
                    └──────┬──────┘
                           │ configure event
                           ▼
                    ┌─────────────┐
                    │  Configured │
                    └──────┬──────┘
                           │ apply_wallpaper()
                           ▼
                    ┌─────────────┐
        pause() ◄───│   Playing   │───► resume()
                    └──────┬──────┘
                           │ clear_wallpaper()
                           ▼
                    ┌─────────────┐
                    │   Stopped   │
                    └──────┬──────┘
                           │ destroy
                           ▼
                    ┌─────────────┐
                    │  Destroyed  │
                    └─────────────┘
```

## Wayland Connection Strategy

### Problem
iced uses its own Wayland connection internally. We need a separate connection for layer-shell surfaces.

### Solution
Create independent Wayland connection in engine thread:

```rust
// Engine creates its own connection
let connection = Connection::connect_to_env()?;
let display = connection.display();
let mut event_queue = connection.new_event_queue();
let registry = display.get_registry(&event_queue.handle(), ());

// Bind required protocols
// - wl_compositor
// - wl_shm
// - zwlr_layer_shell_v1
// - wp_fractional_scale_manager_v1 (optional)
```

This is the standard pattern used by other Wayland overlay/wallpaper tools and works reliably.

## Error Handling

### Engine Errors
- Connection failure: Report to GUI, allow retry
- Surface creation failure: Report specific output error
- MPV initialization failure: Report codec/driver issues

### Recovery Strategy
```rust
match error {
    EngineError::ConnectionLost => {
        // Attempt reconnection after delay
        self.schedule_reconnect();
    }
    EngineError::SurfaceLost(output) => {
        // Recreate surface for specific output
        self.recreate_surface(&output)?;
    }
    EngineError::MpvError(e) => {
        // Log and continue, wallpaper shows black
        tracing::error!("MPV error: {}", e);
    }
}
```

## Configuration

```rust
pub struct EngineConfig {
    /// Default video configuration
    pub video: VideoConfig,
    /// Whether to start playback automatically when wallpaper is applied
    pub auto_play: bool,
    /// FPS limit (None = unlimited/vsync)
    pub fps_limit: Option<u32>,
    /// Pause when on battery power
    pub pause_on_battery: bool,
}
```

## IPC Server (Optional)

When enabled, starts a Unix socket server that accepts `wayvid-ctl` connections:

```rust
pub struct IpcServer {
    listener: UnixListener,
    engine_commands: mpsc::Sender<EngineCommand>,
}

impl IpcServer {
    pub async fn run(&self) {
        loop {
            let (stream, _) = self.listener.accept().await?;
            tokio::spawn(self.handle_client(stream));
        }
    }

    async fn handle_client(&self, stream: UnixStream) {
        // Parse IpcRequest, translate to EngineCommand
        // Execute and return IpcResponse
    }
}
```

## Migration Path

### Phase 1: Engine Core (this proposal)
- Implement PlaybackEngine with basic apply/clear
- Single output support first
- No IPC server yet

### Phase 2: Multi-Output
- Extend to all detected outputs
- Implement output hotplug

### Phase 3: IPC Server
- Add optional IPC server
- Make wayvid-ctl work with GUI

### Phase 4: Polish
- Handle edge cases
- Performance optimization
- Documentation
