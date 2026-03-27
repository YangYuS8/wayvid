# Design: Daemon Architecture Refactoring

## Current Architecture (Problem)

```
┌─────────────────┐       IPC        ┌─────────────────┐
│   wayvid-gui    │ ◄──────────────► │     wayvid      │
│   (Frontend)    │                  │    (Daemon)     │
└─────────────────┘                  └─────────────────┘
        │                                    │
        │ start_daemon_process()             │
        │ tries to spawn "wayvid"            │
        │ binary (DOES NOT EXIST)            │
        ▼                                    ▼
    ❌ FAILS                           NOT BUILT
```

**Problems:**
1. `wayvid` binary was removed in v0.5
2. GUI tries to spawn non-existent process
3. UI shows "Daemon Disconnected" misleadingly
4. Users see errors when clicking "Start Daemon"

## Target Architecture (Solution)

```
┌─────────────────────────────────────────────┐
│                 wayvid-gui                  │
│  ┌───────────────┐  ┌───────────────────┐  │
│  │     UI        │  │  Playback Engine  │  │
│  │   (iced)      │  │  (mpv + wayland)  │  │
│  │               │  │                   │  │
│  │  Start/Stop ──┼──► Internal Control  │  │
│  │  Engine       │  │                   │  │
│  └───────────────┘  └───────────────────┘  │
└─────────────────────────────────────────────┘
```

**Benefits:**
1. Single binary architecture
2. Direct internal communication
3. No IPC for engine control
4. Clearer terminology ("Engine" vs "Daemon")

## Implementation Strategy

### Phase 1: Terminology Rename

Rename daemon-related concepts to engine-related:

| Old Term | New Term |
|----------|----------|
| `daemon_connected` | `engine_running` |
| `StartDaemon` | `StartEngine` |
| `StopDaemon` | `StopEngine` |
| `DaemonStatusUpdated` | `EngineStatusUpdated` |
| `start_daemon_process()` | `start_playback_engine()` |
| `stop_daemon_process()` | `stop_playback_engine()` |

### Phase 2: Logic Refactoring

**Before (ipc.rs):**
```rust
pub async fn start_daemon_process() -> Result<(), String> {
    // Try to start daemon in background
    let result = Command::new("wayvid").spawn();  // ❌ Doesn't exist
    // ...
}
```

**After (ipc.rs):**
```rust
pub async fn start_playback_engine() -> Result<(), String> {
    // Engine is managed internally by GUI
    // This function now initializes the playback subsystem
    // No external process spawning needed
    Ok(())
}
```

### Phase 3: State Management

The engine state will be tracked internally:

```rust
pub struct AppState {
    // ...
    /// Whether the playback engine is running
    pub engine_running: bool,
    
    /// Engine/IPC connection state
    pub ipc_state: ConnectionState,
    // ...
}
```

## IPC Preservation

Note: IPC is still used for **external control** (wayvid-ctl → wayvid-gui), not for internal daemon management. The IPC client/server remains for:

- CLI control via `wayvid-ctl`
- External automation scripts
- Status queries from other processes

## Rollback Plan

If issues arise:
1. Revert message renames
2. Keep IPC infrastructure intact
3. Document that "daemon" terminology refers to internal engine

## Testing Checklist

- [ ] Engine starts without errors
- [ ] Engine stops cleanly
- [ ] Status updates correctly in UI
- [ ] `wayvid-ctl status` still works
- [ ] No crashes on rapid start/stop
