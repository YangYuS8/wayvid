# Change: Refactor Daemon Architecture for v0.5 GUI-First Design

## Why

In wayvid v0.5, the architecture shifted from a standalone daemon (`wayvid`) + separate GUI to a **GUI-first design** where `wayvid-gui` is the main entry point with an **integrated daemon**. However, the codebase still contains legacy code that assumes a separate daemon binary exists:

1. `ipc.rs::start_daemon_process()` tries to spawn `wayvid` binary (which no longer exists)
2. `ipc.rs::stop_daemon_process()` sends IPC quit command but doesn't handle the integrated scenario
3. GUI sidebar has "Start/Stop Daemon" buttons that don't work correctly
4. The concept of "daemon_connected" is misleading when GUI IS the daemon

This creates:
- Runtime errors when users click "Start Daemon" button
- Confusing UI showing "Daemon Disconnected" when it's actually running
- Dead code paths that can never succeed

## What Changes

- **BREAKING**: Remove external daemon launch logic from `ipc.rs`
- **BREAKING**: Change "daemon status" UI to show "playback engine status"
- Refactor `start_daemon_process()` to initialize internal playback engine
- Refactor `stop_daemon_process()` to stop internal playback engine
- Update Messages: `StartDaemon`/`StopDaemon` → `StartEngine`/`StopEngine`
- Update translations for new terminology
- Remove or deprecate IPC-based daemon control (GUI manages engine directly)

## Impact

- **Affected specs**: `gui-integration`
- **Affected code**:
  - `crates/wayvid-gui/src/ipc.rs` - Remove external process spawning
  - `crates/wayvid-gui/src/app.rs` - Update daemon control logic
  - `crates/wayvid-gui/src/messages.rs` - Rename messages
  - `crates/wayvid-gui/src/state.rs` - Rename `daemon_connected` to `engine_running`
  - `crates/wayvid-gui/locales/*.json` - Update translations

## Migration Notes

Users who relied on:
- `wayvid run` command → Use `wayvid-gui --minimized` instead
- Separate daemon process → GUI now manages playback internally
- systemd `wayvid.service` → Updated to use `wayvid-gui --minimized`
