# Change: Implement GUI-Daemon IPC Communication

## Why

The wayvid-gui currently has stubbed IPC functionality that cannot actually communicate with the wayvid daemon. The `IpcClient` in `crates/wayvid-gui/src/ipc.rs` is marked as `dead_code` and all methods simply log messages without performing actual communication. This means:

- Users cannot apply wallpapers from the GUI
- The daemon status shown in the sidebar is always mock data
- Start/Stop daemon buttons don't work properly
- Monitor wallpaper assignments cannot be made

Without functional IPC, the GUI is essentially a non-functional shell that can browse wallpapers but cannot control them.

## What Changes

### Core Implementation
- **IpcClient**: Implement actual Unix socket communication using `tokio::net::UnixStream`
- **Connection Management**: Add reconnection logic and connection state tracking
- **Message Serialization**: Use the existing `IpcRequest`/`IpcResponse` types from `wayvid-core/src/ipc.rs`
- **Subscription Integration**: Create iced subscriptions for async IPC events

### GUI Integration
- Connect `Message::ApplyWallpaper` to `IpcClient::apply_wallpaper`
- Connect `Message::StartDaemon`/`Message::StopDaemon` to actual process management
- Implement real-time daemon status polling via subscription
- Update `Message::RefreshMonitors` to query actual outputs from daemon

### Background Service
- Integrate `BackgroundService` from `service.rs` for daemon process management
- Remove `#![allow(dead_code)]` from `ipc.rs` and `service.rs`

## Impact

### Affected Specs
- None (new capability)

### Affected Code
- `crates/wayvid-gui/src/ipc.rs` - Main IPC client implementation
- `crates/wayvid-gui/src/service.rs` - Background service integration
- `crates/wayvid-gui/src/app.rs` - Message handler updates
- `crates/wayvid-gui/src/state.rs` - Add IPC connection state
- `crates/wayvid-gui/src/messages.rs` - Add IPC-related messages

### Dependencies
- Requires `tokio` (already present)
- Uses `wayvid-core::ipc` types (already defined)

## Success Criteria

1. GUI can connect to running daemon and display real status
2. Applying a wallpaper from GUI actually sets the wallpaper
3. Daemon start/stop buttons work correctly
4. Monitor list shows actual connected monitors
5. Connection loss is handled gracefully with reconnection attempts
