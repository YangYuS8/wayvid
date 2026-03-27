# Tasks: Refactor Daemon Architecture

## Status
- [x] Phase 1: Core Refactoring
- [x] Phase 2: UI Updates
- [x] Phase 3: Translation Updates
- [x] Phase 4: Testing & Cleanup

---

## 1. Core Refactoring

### 1.1 Update Messages
- [x] 1.1.1 Rename `Message::StartDaemon` to `Message::StartEngine` in `messages.rs`
- [x] 1.1.2 Rename `Message::StopDaemon` to `Message::StopEngine` in `messages.rs`
- [x] 1.1.3 Rename `Message::DaemonStatusUpdated` to `Message::EngineStatusUpdated` in `messages.rs`

### 1.2 Update State
- [x] 1.2.1 Rename `AppState::daemon_connected` to `AppState::engine_running` in `state.rs`
- [x] 1.2.2 Update all references to `daemon_connected` throughout codebase
- [x] 1.2.3 Update `ConnectionState` enum documentation

### 1.3 Refactor IPC Functions
- [x] 1.3.1 Remove external process spawning from `start_daemon_process()`
- [x] 1.3.2 Rename `start_daemon_process()` to `start_playback_engine()`
- [x] 1.3.3 Implement internal engine initialization logic
- [x] 1.3.4 Rename `stop_daemon_process()` to `stop_playback_engine()`
- [x] 1.3.5 Implement internal engine shutdown logic
- [x] 1.3.6 Update function exports in `ipc.rs`

### 1.4 Update App Logic
- [x] 1.4.1 Update `Message::StartEngine` handler in `app.rs`
- [x] 1.4.2 Update `Message::StopEngine` handler in `app.rs`
- [x] 1.4.3 Update `Message::EngineStatusUpdated` handler in `app.rs`
- [x] 1.4.4 Update sidebar daemon status display

---

## 2. UI Updates

### 2.1 Sidebar Updates
- [x] 2.1.1 Change "Daemon Running/Stopped" to "Engine Running/Stopped"
- [x] 2.1.2 Update daemon control button labels
- [x] 2.1.3 Update status indicator colors and text

### 2.2 Settings View
- [x] 2.2.1 Update any daemon-related settings labels (N/A - no daemon settings)
- [x] 2.2.2 Update settings descriptions (N/A - no daemon settings)

---

## 3. Translation Updates

### 3.1 English Translations
- [x] 3.1.1 Update `locales/en.toml` with new engine terminology
- [x] 3.1.2 Remove deprecated daemon keys

### 3.2 Chinese Translations
- [x] 3.2.1 Update `locales/zh-CN.toml` with new engine terminology
- [x] 3.2.2 Remove deprecated daemon keys

---

## 4. Testing & Cleanup

### 4.1 Code Cleanup
- [x] 4.1.1 Remove dead code related to external daemon spawning
- [x] 4.1.2 Update code comments to reflect new architecture
- [x] 4.1.3 Update IPC tests if any (N/A - no daemon-specific tests)

### 4.2 Verification
- [x] 4.2.1 Test Start Engine button functionality
- [x] 4.2.2 Test Stop Engine button functionality
- [x] 4.2.3 Test engine status display updates
- [x] 4.2.4 Verify no references to old `wayvid` binary remain in GUI code
- [x] 4.2.5 Run `cargo clippy` to ensure no warnings

### 4.3 Documentation
- [x] 4.3.1 Update inline code documentation
- [x] 4.3.2 Update user-facing documentation if needed (N/A - translations updated)
