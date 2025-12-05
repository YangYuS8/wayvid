# Tasks: Refactor Daemon Architecture

## Status
- [ ] Phase 1: Core Refactoring
- [ ] Phase 2: UI Updates
- [ ] Phase 3: Translation Updates
- [ ] Phase 4: Testing & Cleanup

---

## 1. Core Refactoring

### 1.1 Update Messages
- [ ] 1.1.1 Rename `Message::StartDaemon` to `Message::StartEngine` in `messages.rs`
- [ ] 1.1.2 Rename `Message::StopDaemon` to `Message::StopEngine` in `messages.rs`
- [ ] 1.1.3 Rename `Message::DaemonStatusUpdated` to `Message::EngineStatusUpdated` in `messages.rs`

### 1.2 Update State
- [ ] 1.2.1 Rename `AppState::daemon_connected` to `AppState::engine_running` in `state.rs`
- [ ] 1.2.2 Update all references to `daemon_connected` throughout codebase
- [ ] 1.2.3 Update `ConnectionState` enum documentation

### 1.3 Refactor IPC Functions
- [ ] 1.3.1 Remove external process spawning from `start_daemon_process()`
- [ ] 1.3.2 Rename `start_daemon_process()` to `start_playback_engine()`
- [ ] 1.3.3 Implement internal engine initialization logic
- [ ] 1.3.4 Rename `stop_daemon_process()` to `stop_playback_engine()`
- [ ] 1.3.5 Implement internal engine shutdown logic
- [ ] 1.3.6 Update function exports in `ipc.rs`

### 1.4 Update App Logic
- [ ] 1.4.1 Update `Message::StartEngine` handler in `app.rs`
- [ ] 1.4.2 Update `Message::StopEngine` handler in `app.rs`
- [ ] 1.4.3 Update `Message::EngineStatusUpdated` handler in `app.rs`
- [ ] 1.4.4 Update sidebar daemon status display

---

## 2. UI Updates

### 2.1 Sidebar Updates
- [ ] 2.1.1 Change "Daemon Running/Stopped" to "Engine Running/Stopped"
- [ ] 2.1.2 Update daemon control button labels
- [ ] 2.1.3 Update status indicator colors and text

### 2.2 Settings View
- [ ] 2.2.1 Update any daemon-related settings labels
- [ ] 2.2.2 Update settings descriptions

---

## 3. Translation Updates

### 3.1 English Translations
- [ ] 3.1.1 Update `locales/en.json` with new engine terminology
- [ ] 3.1.2 Remove deprecated daemon keys

### 3.2 Chinese Translations
- [ ] 3.2.1 Update `locales/zh-CN.json` with new engine terminology
- [ ] 3.2.2 Remove deprecated daemon keys

---

## 4. Testing & Cleanup

### 4.1 Code Cleanup
- [ ] 4.1.1 Remove dead code related to external daemon spawning
- [ ] 4.1.2 Update code comments to reflect new architecture
- [ ] 4.1.3 Update IPC tests if any

### 4.2 Verification
- [ ] 4.2.1 Test Start Engine button functionality
- [ ] 4.2.2 Test Stop Engine button functionality
- [ ] 4.2.3 Test engine status display updates
- [ ] 4.2.4 Verify no references to old `wayvid` binary remain in GUI code
- [ ] 4.2.5 Run `cargo clippy` to ensure no warnings

### 4.3 Documentation
- [ ] 4.3.1 Update inline code documentation
- [ ] 4.3.2 Update user-facing documentation if needed
