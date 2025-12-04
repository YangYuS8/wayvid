# implement-settings-persistence - Tasks

## 1. Enable and Initialize Settings Module
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 1.1 Remove `#![allow(dead_code)]` from `settings.rs`
2. [ ] 1.2 Export `AppSettings`, `SettingsManager`, `AutostartManager` from module
3. [ ] 1.3 Update `AppState` to use `settings::AppSettings` instead of `state::Settings`
4. [ ] 1.4 Initialize `SettingsManager` in application startup
5. [ ] 1.5 Load saved settings and apply to initial state
6. [ ] 1.6 Handle missing/corrupted settings file gracefully

### Acceptance Criteria
- Settings load from disk on startup
- Default settings used if file doesn't exist
- Corrupted files don't crash the app

---

## 2. Connect Settings UI to Persistence
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 2.1 Update `Message::ToggleAutostart` to call `AutostartManager::set_enabled()`
2. [ ] 2.2 Update `Message::ToggleMinimizeToTray` to persist setting
3. [ ] 2.3 Update `Message::TogglePauseOnBattery` to persist setting
4. [ ] 2.4 Update `Message::TogglePauseOnFullscreen` to persist setting
5. [ ] 2.5 Update `Message::VolumeChanged` to persist setting
6. [ ] 2.6 Update `Message::FpsLimitChanged` to persist setting
7. [ ] 2.7 Update `Message::LanguageChanged` to persist setting
8. [ ] 2.8 Trigger auto-save after each setting change

### Acceptance Criteria
- Each setting change is persisted to disk
- Changes survive application restart

---

## 3. Implement Auto-save with Debouncing
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 3.1 Create tokio task for debounced saving (500ms delay)
2. [ ] 3.2 Add channel for save requests in AppState
3. [ ] 3.3 Batch multiple changes within debounce window
4. [ ] 3.4 Handle save errors gracefully (log, don't crash)
5. [ ] 3.5 Force save on application close

### Acceptance Criteria
- Rapid setting changes don't cause excessive disk writes
- Settings are saved even if app closes quickly

---

## 4. Autostart Manager Integration
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 4.1 Check current autostart state on settings view load
2. [ ] 4.2 Update toggle to reflect actual XDG autostart status
3. [ ] 4.3 Call `AutostartManager::enable()` when toggled on
4. [ ] 4.4 Call `AutostartManager::disable()` when toggled off
5. [ ] 4.5 Handle permission errors (show user-friendly message)
6. [ ] 4.6 Verify desktop entry contents match current binary path

### Acceptance Criteria
- Autostart toggle reflects actual system state
- Desktop entry is created/removed correctly

---

## 5. Language Setting Persistence
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 5.1 Load language setting before GUI initialization
2. [ ] 5.2 Apply language via `i18n::set_language()` at startup
3. [ ] 5.3 Handle "System" language option (detect from locale)
4. [ ] 5.4 Persist language changes immediately
5. [ ] 5.5 Show restart hint when language changes (if needed)

### Acceptance Criteria
- Language preference persists across restarts
- Correct language is applied at startup

---

## 6. Power Management Integration
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 6.1 Create power monitoring subscription
2. [ ] 6.2 Poll `PowerMonitor::is_on_battery()` periodically (every 30s)
3. [ ] 6.3 Add `Message::PowerStateChanged(bool)` for battery status
4. [ ] 6.4 Pause playback when on battery (if setting enabled)
5. [ ] 6.5 Resume playback when on AC power
6. [ ] 6.6 Implement fullscreen detection (compositor-specific TODO)

### Acceptance Criteria
- Playback pauses automatically on battery (if enabled)
- Battery state changes are detected reliably

---

## 7. Window State Persistence
**Status**: Not Started
**Priority**: Low

### Steps
1. [ ] 7.1 Add window size fields to GuiSettings
2. [ ] 7.2 Save window size on close (via window events)
3. [ ] 7.3 Restore window size on startup via `window_size()` 
4. [ ] 7.4 Validate saved dimensions are reasonable
5. [ ] 7.5 Handle multi-monitor scenarios gracefully

### Acceptance Criteria
- Window size persists across restarts
- Invalid saved sizes don't cause issues

---

## 8. Command Line Flags
**Status**: Not Started
**Priority**: Low

### Steps
1. [ ] 8.1 Add `--minimized` flag to start minimized
2. [ ] 8.2 Add `--config <path>` flag for custom config location
3. [ ] 8.3 Check `start_minimized` setting if no flag provided
4. [ ] 8.4 Update desktop entry to use `--minimized` for autostart

### Acceptance Criteria
- App can start minimized via flag or setting
- Custom config path is respected

---

## 9. Testing and Validation
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 9.1 Test settings file creation on first run
2. [ ] 9.2 Test settings persistence across restarts
3. [ ] 9.3 Test autostart enable/disable
4. [ ] 9.4 Test language persistence
5. [ ] 9.5 Test corrupted settings file recovery
6. [ ] 9.6 Test power management auto-pause

### Acceptance Criteria
- All persistence features work reliably
- Edge cases are handled gracefully

---

## Progress Tracking

| Task | Status | Blockers |
|------|--------|----------|
| Enable Settings Module | Not Started | None |
| Connect UI to Persistence | Not Started | Task 1 |
| Auto-save Debouncing | Not Started | Task 1 |
| Autostart Manager | Not Started | Task 1 |
| Language Persistence | Not Started | Task 2 |
| Power Management | Not Started | Task 2 |
| Window State | Not Started | Task 1 |
| Command Line Flags | Not Started | None |
| Testing | Not Started | All above |

## Technical Notes

### Settings File Location
```
~/.config/wayvid/settings.yaml
```

### Settings YAML Format
```yaml
gui:
  window_width: 1200
  window_height: 800
  minimize_to_tray: true
  start_minimized: false
  theme: dark
  language: system

playback:
  volume: 0.0
  fps_limit: null
  preferred_monitor: null
  loop_mode: true
  shuffle: false

autostart:
  enabled: false
  restore_last_wallpaper: true
  monitor_states: []

power:
  pause_on_battery: true
  pause_on_fullscreen: true
  battery_fps_limit: 15

library:
  folders: []
  workshop_enabled: true
  steam_library_paths: []
  thumbnail_size: 256
```

### XDG Autostart Location
```
~/.config/autostart/wayvid.desktop
```

### Desktop Entry Content
```desktop
[Desktop Entry]
Type=Application
Name=Wayvid
Comment=Animated wallpaper manager for Wayland
Exec=wayvid-gui --minimized
Icon=wayvid
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
```
