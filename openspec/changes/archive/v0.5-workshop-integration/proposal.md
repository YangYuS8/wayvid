# v0.5 Workshop Integration & Installation

## Summary
Complete the GUI-first experience by integrating Steam Workshop wallpapers, improving local folder support, and creating proper installation/uninstallation scripts.

## Motivation
1. Users need a one-click installation experience for testing
2. Wallpaper Engine Workshop is the primary wallpaper source for most users
3. GUI should be discoverable from application launchers
4. Users need control over the wallpaper daemon without config files

## Tasks

### Task 1: Installation/Uninstallation Scripts
**File**: `scripts/install.sh`, `scripts/uninstall.sh`

Create scripts that:
- Build release binaries
- Install to `~/.local/bin/` (user) or `/usr/local/bin/` (system)
- Install desktop file to proper XDG location
- Install systemd user service (optional)
- Handle icon installation
- Support `--user` and `--system` modes

### Task 2: Workshop Integration in GUI
**Files**: `crates/wayvid-gui/src/` views and state

Integrate existing `wayvid-library::WorkshopScanner`:
- Scan Steam Workshop on startup (background)
- Display wallpaper preview images (project's `preview` field)
- Show Workshop wallpapers prominently in Library view
- Separate tab/filter for Workshop vs Local wallpapers
- Handle missing Steam/Wallpaper Engine gracefully

### Task 3: Local Folder Support (Secondary Source)
**Files**: `crates/wayvid-gui/src/views/folders.rs`

Enhance folder management:
- Functional "Add Folder" dialog
- Scan folders for video/image files
- Display local wallpapers alongside Workshop
- Persist folder list in settings

### Task 4: Desktop Integration
**Files**: `packaging/wayvid-gui.desktop`, icon files

Ensure:
- Desktop file is installed correctly
- Icon is available (SVG or PNG)
- StartupWMClass matches application window
- Categories are appropriate

### Task 5: Daemon Control from GUI
**Files**: `crates/wayvid-gui/src/` 

Add controls to:
- Start/stop wayvid daemon
- Show daemon status (running/stopped)
- Auto-start daemon when applying wallpaper
- Handle daemon lifecycle gracefully

### Task 6: Compositor Conflict Avoidance
**Design Decision**:
- Do NOT modify niri/hyprland config files
- Use IPC socket for daemon communication
- Document recommended autostart configuration
- Provide example configs in `configs/` (already exists)

## Implementation Order
1. Installation scripts (enables testing)
2. Desktop file verification
3. Workshop integration in GUI
4. Local folder improvements
5. Daemon control UI

## Technical Notes

### Installation Paths
```
User install (~/.local/):
  ~/.local/bin/wayvid
  ~/.local/bin/wayvid-gui
  ~/.local/bin/wayvid-ctl
  ~/.local/share/applications/wayvid-gui.desktop
  ~/.local/share/icons/hicolor/scalable/apps/wayvid.svg

System install (/usr/local/):
  /usr/local/bin/wayvid
  /usr/local/bin/wayvid-gui
  /usr/local/bin/wayvid-ctl
  /usr/share/applications/wayvid-gui.desktop
  /usr/share/icons/hicolor/scalable/apps/wayvid.svg
```

### Workshop Preview Loading
```rust
// Already implemented in wayvid-library
if let Some(preview) = project.preview_image(item_path) {
    if preview.exists() {
        item.thumbnail_path = Some(preview);
    }
}
```

### Daemon Communication
Use existing `wayvid-ctl` IPC protocol for:
- Status checking
- Start/stop commands
- Wallpaper application

## Non-Goals
- Online Workshop browsing (future)
- Config file modification for compositors
- System service installation (only user service)

## Testing
1. Run `scripts/install.sh --user`
2. Launch from application menu
3. Verify Workshop wallpapers appear
4. Test wallpaper application
5. Run `scripts/uninstall.sh --user`
