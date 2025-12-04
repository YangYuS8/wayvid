# implement-gui-daemon-ipc - Tasks

## 1. IPC Client Implementation
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 1.1 Remove `#![allow(dead_code)]` from `ipc.rs`
2. [ ] 1.2 Add `tokio::net::UnixStream` based socket connection
3. [ ] 1.3 Implement `connect()` method with timeout
4. [ ] 1.4 Implement `send_request()` with JSON serialization
5. [ ] 1.5 Implement `recv_response()` with JSON deserialization
6. [ ] 1.6 Update `apply_wallpaper()` to send `IpcRequest::Apply`
7. [ ] 1.7 Update `pause()`, `resume()`, `stop()` methods
8. [ ] 1.8 Update `status()` to parse `IpcResponse::Status`
9. [ ] 1.9 Add connection state tracking (Connected/Disconnected/Error)
10. [ ] 1.10 Implement reconnection with exponential backoff

### Acceptance Criteria
- IpcClient can establish Unix socket connection
- All IPC methods send proper requests and parse responses
- Connection failures are handled gracefully

---

## 2. IPC Subscription for Real-time Updates
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 2.1 Create `ipc_subscription()` function returning `Subscription<Message>`
2. [ ] 2.2 Implement periodic status polling (every 2 seconds)
3. [ ] 2.3 Add `Message::IpcEvent` variants for connection state changes
4. [ ] 2.4 Add `Message::DaemonStatusReceived(DaemonStatus)` message
5. [ ] 2.5 Integrate subscription in `App::subscription()`

### Acceptance Criteria
- GUI receives real-time daemon status updates
- Connection state changes are reflected in UI immediately

---

## 3. App Message Handler Updates
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 3.1 Update `Message::ApplyWallpaper` handler to use IpcClient
2. [ ] 3.2 Update `Message::ApplyToMonitor` handler to use IpcClient
3. [ ] 3.3 Update `Message::ClearMonitor` handler to use IpcClient
4. [ ] 3.4 Update `Message::RefreshMonitors` to query daemon via IPC
5. [ ] 3.5 Handle `Message::IpcEvent` for connection state updates
6. [ ] 3.6 Handle `Message::DaemonStatusReceived` for status updates

### Acceptance Criteria
- All wallpaper operations use actual IPC communication
- Monitor list reflects real connected outputs

---

## 4. Background Service Integration
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 4.1 Remove `#![allow(dead_code)]` from `service.rs`
2. [ ] 4.2 Add `BackgroundService` to `AppState`
3. [ ] 4.3 Update `Message::StartDaemon` to start wayvid process
4. [ ] 4.4 Update `Message::StopDaemon` to send `IpcRequest::Quit`
5. [ ] 4.5 Implement daemon process monitoring
6. [ ] 4.6 Add service status to sidebar display

### Acceptance Criteria
- GUI can start and stop the daemon process
- Daemon status is accurately reflected in sidebar

---

## 5. State Management Updates
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 5.1 Add `IpcConnectionState` enum to state.rs
2. [ ] 5.2 Add `ipc_state: IpcConnectionState` to AppState
3. [ ] 5.3 Add `last_ipc_error: Option<String>` to AppState
4. [ ] 5.4 Update sidebar to show connection state with icon
5. [ ] 5.5 Show error tooltip on connection failure

### Acceptance Criteria
- Connection state is visible to user
- Errors are displayed appropriately

---

## 6. Testing and Validation
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 6.1 Test GUI with daemon running - wallpaper application works
2. [ ] 6.2 Test GUI with daemon not running - graceful handling
3. [ ] 6.3 Test daemon start/stop from GUI
4. [ ] 6.4 Test reconnection after daemon restart
5. [ ] 6.5 Verify no deadlocks or race conditions

### Acceptance Criteria
- All IPC operations work correctly
- Error states are handled gracefully
- No blocking or freezing of GUI

---

## Progress Tracking

| Task | Status | Blockers |
|------|--------|----------|
| IPC Client Implementation | Not Started | None |
| IPC Subscription | Not Started | Task 1 |
| App Handler Updates | Not Started | Task 1, 2 |
| Background Service | Not Started | Task 1 |
| State Management | Not Started | None |
| Testing | Not Started | All above |

## Technical Notes

### Socket Path
Use `wayvid_core::ipc::default_socket_path()` for consistency:
- Primary: `$XDG_RUNTIME_DIR/wayvid.sock`
- Fallback: `/tmp/wayvid-$USER.sock`

### Message Format
JSON-based protocol defined in `wayvid_core::ipc`:
```rust
// Request
IpcRequest::Apply { path, output, mode }
IpcRequest::Status
IpcRequest::Outputs

// Response
IpcResponse::Ok { message }
IpcResponse::Status { running, version, outputs }
IpcResponse::Outputs { outputs }
```

### Async Considerations
- Use `tokio::spawn` for non-blocking IPC calls
- Subscription should not block the UI thread
- Consider using a dedicated IPC task with channel communication
