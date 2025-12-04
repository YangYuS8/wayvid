# Change: Implement Settings Persistence

## Why

The wayvid-gui has a comprehensive settings system defined in `crates/wayvid-gui/src/settings.rs` that is marked as `dead_code` and not integrated. Currently:

- User settings (volume, FPS limit, pause behavior) are not saved
- Language preference resets on each launch
- Autostart cannot be configured
- Power management settings have no effect
- Window size and position are not remembered

The `AppSettings`, `SettingsManager`, and `AutostartManager` classes exist but aren't connected to the GUI, making user customization pointless.

## What Changes

### Settings Integration
- **Enable settings.rs**: Remove `#![allow(dead_code)]` and integrate module
- **Load on Startup**: Load settings from `~/.config/wayvid/settings.yaml`
- **Auto-save**: Use `SettingsManager` with debounced auto-save
- **Settings View**: Connect settings UI controls to persistent settings

### Autostart Management
- **XDG Autostart**: Integrate `AutostartManager` for desktop entry management
- **Toggle Control**: Connect autostart toggle in settings view
- **Service Mode**: Support `--minimized` flag for startup minimized

### Power Management
- **Battery Detection**: Use `PowerMonitor::is_on_battery()` for battery state
- **Fullscreen Detection**: Implement compositor-specific fullscreen detection
- **Auto-pause**: Pause playback based on power settings

### Window State
- **Remember Size**: Save window dimensions on close
- **Remember Position**: Save window position (compositor permitting)
- **Start Minimized**: Support starting minimized to tray

## Impact

### Affected Specs
- None (new capability)

### Affected Code
- `crates/wayvid-gui/src/settings.rs` - Enable and extend
- `crates/wayvid-gui/src/app.rs` - Initialize and use SettingsManager
- `crates/wayvid-gui/src/state.rs` - Connect state.settings to persistence
- `crates/wayvid-gui/src/views/settings.rs` - Update UI bindings
- `crates/wayvid-gui/src/main.rs` - Add command line flags

### Dependencies
- `serde_yaml` - Already present for config parsing
- `dirs` - Already present for XDG paths

## Success Criteria

1. Settings persist across application restarts
2. Autostart toggle creates/removes XDG desktop entry
3. Language setting persists and applies on startup
4. Power management settings affect playback behavior
5. Window size is remembered between sessions
