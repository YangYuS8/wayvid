## 1. Engine Core API

### 1.1 PlaybackEngine Interface
- [x] Create `PlaybackEngine` struct in `wayvid-engine/src/engine/mod.rs`
- [x] Define engine configuration struct `EngineConfig`
- [x] Implement `spawn_engine(config: EngineConfig) -> Result<(EngineHandle, Receiver<EngineEvent>)>`
- [x] Implement engine thread with calloop event loop
- [x] Implement shutdown via `EngineHandle::request_shutdown()`
- [x] Implement `ApplyWallpaper` command handler
- [x] Implement `ClearWallpaper` command handler
- [x] Implement `PlaybackEngine::get_outputs() -> Vec<OutputInfo>` (via OutputsList event)
- [x] Define `EngineStatus` enum

### 1.2 WallpaperSession Management
- [x] Create `WallpaperSession` struct for per-output playback
- [x] Implement session lifecycle: create, start, pause, resume, destroy
- [x] Connect `MpvPlayer` with `LayerSurface` for rendering
- [ ] Implement shared decoder optimization for same-source outputs (deferred)

### 1.3 Wayland Backend Completion
- [x] Implement Wayland registry event handling (wl_registry Dispatch)
- [x] Implement wl_output binding and event processing (Geometry, Mode, Scale, Name, Done)
- [x] Complete `LayerSurface` implementation using zwlr_layer_shell_v1 directly
- [x] Implement configure event handling for ZwlrLayerSurfaceV1
- [x] Implement surface commit and frame callback
- [x] Add EGL surface integration (EglContext, EglWindow, create_window, destroy_surface)
- [x] Implement output hotplug handling (via GlobalRemove event)

## 2. GUI Integration

### 2.1 Engine Embedding
- [x] Add `EngineController` wrapper in `wayvid-gui/src/engine.rs`
- [x] Add `EngineController` as field in `App` struct
- [x] Create engine initialization in `App::new()`
- [x] Implement engine shutdown on app exit (via Drop or explicit stop)

### 2.2 Message Handlers
- [x] Refactor `Message::StartEngine` to initialize embedded engine
- [x] Refactor `Message::StopEngine` to stop embedded engine
- [x] Refactor `Message::ApplyToMonitor` to call engine directly (with IPC fallback)
- [x] Refactor `Message::ClearMonitor` to call engine directly (with IPC fallback)
- [x] Add `Message::EngineEvent` for engine -> GUI communication
- [x] Add `Message::PollEngineEvents` for polling engine events
- [x] Implement engine event polling subscription

### 2.3 State Management
- [x] Replace IPC-based status polling with engine callbacks (EngineEvent)
- [x] Implement engine status subscription (engine_subscription)
- [x] Sync monitor list from engine's Wayland backend (OutputsList event)
- [x] Update `engine_running` based on actual engine state

## 3. Built-in IPC Server (Optional)

### 3.1 Server Implementation
- [x] Create `IpcServer` struct in `wayvid-gui/src/ipc_server.rs`
- [x] Implement Unix socket listener
- [x] Handle `IpcRequest` messages and call engine methods
- [x] Send `IpcResponse` back to clients

### 3.2 Integration
- [ ] Start IPC server when engine starts (deferred to future iteration)
- [ ] Stop IPC server when engine stops (deferred to future iteration)
- [ ] Make IPC server optional (configurable) (deferred to future iteration)

## 4. Cleanup & Migration

### 4.1 Remove Daemon Dependencies
- [x] Remove `start_daemon_process()` stub code (already updated in v0.5)
- [x] Clean up external process spawning logic (no longer needed)
- [x] Update error messages and user feedback

### 4.2 Update IPC Client
- [x] Modify `ipc_subscription` to work with built-in server (updated)
- [x] Update `get_monitors_ipc` to prefer engine data over wlr-randr
- [x] Keep wlr-randr as fallback when engine not running

### 4.3 Documentation
- [ ] Update README with new architecture
- [ ] Update user guide documentation
- [ ] Add developer documentation for engine API

## 5. Testing & Validation

### 5.1 Manual Testing
- [x] Verify engine starts with GUI
- [x] Verify wallpaper applies to correct output
- [ ] Verify multi-monitor support
- [x] Verify engine stops cleanly on exit
- [ ] Verify `wayvid-ctl` can control GUI via IPC

### 5.2 Edge Cases
- [x] Test output hotplug (connect/disconnect monitor) - implemented via GlobalRemove
- [ ] Test engine restart after crash
- [x] Test concurrent wallpaper operations (wallpaper switching works)
- [ ] Test memory cleanup on repeated start/stop

## 6. Bug Fixes (Completed)

### 6.1 EGL Surface Management
- [x] Fix EGL surface resource leak on wallpaper switch
- [x] Add `EglContext::destroy_surface()` method
- [x] Add `WallpaperSession::cleanup_egl()` method
- [x] Add `EglContext::make_current_none()` method

### 6.2 Layer Surface Configuration
- [x] Add `set_anchor()` call for full-screen coverage
- [x] Set `exclusive_zone = -1` to cover top bars (noctalia-shell)
- [x] Handle compositor-scaled dimensions correctly

### 6.3 Wallpaper Transition Optimization
- [x] Implement hot-swap optimization (reuse layer surface when switching)
- [x] Add `WallpaperSession::load_new_wallpaper()` for seamless transitions
- [x] Only swap buffers when MPV has a valid frame (reduce flicker)
