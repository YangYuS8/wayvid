## 1. Engine Core API

### 1.1 PlaybackEngine Interface
- [ ] Create `PlaybackEngine` struct in `wayvid-engine/src/engine.rs`
- [ ] Define engine configuration struct `EngineConfig`
- [ ] Implement `PlaybackEngine::new(config: EngineConfig) -> Result<Self>`
- [ ] Implement `PlaybackEngine::start() -> Result<()>` - Start Wayland event loop
- [ ] Implement `PlaybackEngine::stop() -> Result<()>` - Stop all playback
- [ ] Implement `PlaybackEngine::apply_wallpaper(path, output) -> Result<()>`
- [ ] Implement `PlaybackEngine::clear_wallpaper(output) -> Result<()>`
- [ ] Implement `PlaybackEngine::get_outputs() -> Vec<OutputInfo>`
- [ ] Implement `PlaybackEngine::get_status() -> EngineStatus`

### 1.2 WallpaperSession Management
- [ ] Create `WallpaperSession` struct for per-output playback
- [ ] Implement session lifecycle: create, start, pause, resume, destroy
- [ ] Connect `MpvPlayer` with `LayerSurface` for rendering
- [ ] Implement shared decoder optimization for same-source outputs

### 1.3 Wayland Backend Completion
- [ ] Complete `LayerSurface::new()` implementation (migrate from AUR code)
- [ ] Implement configure event handling
- [ ] Implement surface commit and damage tracking
- [ ] Add EGL surface integration
- [ ] Implement output hotplug handling

## 2. GUI Integration

### 2.1 Engine Embedding
- [ ] Add `PlaybackEngine` as optional field in `App` struct
- [ ] Create engine initialization logic in startup sequence
- [ ] Implement engine shutdown on app exit

### 2.2 Message Handlers
- [ ] Refactor `Message::StartEngine` to initialize embedded engine
- [ ] Refactor `Message::StopEngine` to stop embedded engine
- [ ] Refactor `Message::ApplyToMonitor` to call engine directly
- [ ] Refactor `Message::ClearMonitor` to call engine directly
- [ ] Add `Message::EngineEvent` for engine -> GUI communication

### 2.3 State Management
- [ ] Replace IPC-based status polling with engine callbacks
- [ ] Implement engine status subscription
- [ ] Sync monitor list from engine's Wayland backend
- [ ] Update `engine_running` based on actual engine state

## 3. Built-in IPC Server (Optional)

### 3.1 Server Implementation
- [ ] Create `IpcServer` struct in `wayvid-gui/src/ipc_server.rs`
- [ ] Implement Unix socket listener
- [ ] Handle `IpcRequest` messages and call engine methods
- [ ] Send `IpcResponse` back to clients

### 3.2 Integration
- [ ] Start IPC server when engine starts
- [ ] Stop IPC server when engine stops
- [ ] Make IPC server optional (configurable)

## 4. Cleanup & Migration

### 4.1 Remove Daemon Dependencies
- [ ] Remove `start_daemon_process()` stub code
- [ ] Clean up external process spawning logic
- [ ] Update error messages and user feedback

### 4.2 Update IPC Client
- [ ] Modify `ipc_subscription` to work with built-in server
- [ ] Update `get_monitors_ipc` to prefer engine data over wlr-randr
- [ ] Keep wlr-randr as fallback when engine not running

### 4.3 Documentation
- [ ] Update README with new architecture
- [ ] Update user guide documentation
- [ ] Add developer documentation for engine API

## 5. Testing & Validation

### 5.1 Manual Testing
- [ ] Verify engine starts with GUI
- [ ] Verify wallpaper applies to correct output
- [ ] Verify multi-monitor support
- [ ] Verify engine stops cleanly on exit
- [ ] Verify `wayvid-ctl` can control GUI via IPC

### 5.2 Edge Cases
- [ ] Test output hotplug (connect/disconnect monitor)
- [ ] Test engine restart after crash
- [ ] Test concurrent wallpaper operations
- [ ] Test memory cleanup on repeated start/stop
