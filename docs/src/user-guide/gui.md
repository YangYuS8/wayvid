# GUI Control Panel

wayvid-gui provides an intuitive graphical interface for managing video wallpapers, redesigned with a Wallpaper Engine-inspired layout for streamlined workflow.

## Starting the GUI

```bash
wayvid-gui
```

The GUI automatically connects to the daemon when running. It functions as:
- 🖼️ **Wallpaper Manager** - Browse and apply wallpapers
- 🎛️ **Control Panel** - Runtime playback control via IPC
- 📊 **Status Monitor** - Real-time daemon status
- 🔧 **Configuration Editor** - Visual config editing

## Interface Overview

The new interface follows a Wallpaper Engine-inspired design:

```
┌─────────────────────────────────────────────────────────────┐
│  [Wallpapers]  [Settings]                        [🌐 EN/中] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐      │
│   │  Video  │  │  Video  │  │  Video  │  │  Video  │      │
│   │   1     │  │   2     │  │   3     │  │   4     │      │
│   │ [✓]     │  │         │  │         │  │         │      │
│   └─────────┘  └─────────┘  └─────────┘  └─────────┘      │
│                                                             │
│   ┌─────────┐  ┌─────────┐  ┌─────────┐                   │
│   │Workshop │  │Workshop │  │Workshop │  [+ Add]          │
│   │  Item   │  │  Item   │  │  Item   │                   │
│   └─────────┘  └─────────┘  └─────────┘                   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │  DP-1    │  │ HDMI-A-1 │  │  eDP-1   │   ← Monitor Bar │
│  │  [✓]     │  │          │  │          │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Elements

1. **Bottom Monitor Selector**: Quick access to all connected displays
2. **Unified Wallpaper Library**: All sources (local, workshop) in one grid
3. **Click-to-Apply**: Single click to select, double-click to apply
4. **Simplified Tabs**: Just "Wallpapers" and "Settings"

## Daemon Management

### Connection Status (Top Panel)

| Status | Meaning |
|--------|---------|
| 🟢 **Connected** | GUI is connected to daemon |
| 🟡 **Connecting...** | Attempting connection |
| ⚪ **Disconnected** | Daemon not running |
| 🔴 **Error** | Connection failed |

### Control Buttons

**When daemon is running:**
- ⏹ **Stop Daemon** - Stop wayvid service
- 🔄 **Restart** - Restart daemon

**When daemon is NOT running:**
- 🚀 **Start Daemon** - Start wayvid via systemd

## Features

### 🖼️ Wallpapers Tab (Unified Library)

The main view combines all wallpaper sources:

| Source Type | Icon | Description |
|-------------|------|-------------|
| Local File | 📁 | Video files from filesystem |
| Directory | 📂 | Folders (playlist mode) |
| URL | 🌐 | HTTP/RTSP streams |
| Workshop | 🎮 | Wallpaper Engine items |

**Interaction:**
- **Single Click**: Select wallpaper (shows info)
- **Double Click**: Apply to selected monitor
- **Drag & Drop**: Drop video files onto window

**Grid Features:**
- Thumbnail previews (when available)
- Source type indicators
- Selection highlight
- Add button for new sources

### ⚙️ Settings Tab

Configure playback and daemon behavior:

**Layout Mode:**
- **Fill** (default): Cover screen, crop if needed
- **Contain**: Fit inside screen (may letterbox)
- **Stretch**: Fill exactly (may distort)
- **Cover**: Alias for Fill
- **Centre**: Original size, centered

**Playback:**
- Loop: Enable/disable video looping
- Volume: 0-100% slider
- Mute: Toggle audio

**Power Management:**
- Battery-aware throttling
- Workspace visibility detection (Niri)

## Bottom Monitor Bar

The bottom panel shows all connected displays:

```
┌──────────────────────────────────────────────────────────────┐
│  ┌────────────┐  ┌────────────┐  ┌────────────┐              │
│  │   DP-1     │  │  HDMI-A-1  │  │   eDP-1    │              │
│  │ 2560x1440  │  │ 1920x1080  │  │ 1920x1200  │              │
│  │    [✓]     │  │            │  │            │              │
│  └────────────┘  └────────────┘  └────────────┘              │
└──────────────────────────────────────────────────────────────┘
```

- **Click** a monitor to select it as target
- **Selected monitor** shows checkmark and highlight
- **Resolution** displayed under name
- **Active source** shown if playing

## Quick Workflow

### Apply Wallpaper (3 Steps)

1. **Select Monitor**: Click target in bottom bar
2. **Browse Wallpapers**: Find desired wallpaper in grid
3. **Apply**: Double-click wallpaper (or single-click + Enter)

### Add New Wallpaper

1. Click **[+ Add]** button in wallpaper grid
2. Choose source type (File, Directory, URL, Workshop)
3. Enter path or select from file browser
4. Wallpaper appears in library

### Multi-Monitor Setup

1. Select first monitor in bottom bar
2. Double-click desired wallpaper
3. Select second monitor
4. Double-click different wallpaper
5. Each monitor plays independently

## Language Support

wayvid-gui supports multiple languages:

- 🇺🇸 English (default)
- 🇨🇳 简体中文

**Auto-detection**: GUI detects system locale on startup.
**Manual switch**: Use language selector (🌐) in top-right corner.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Apply selected wallpaper |
| `Space` | Toggle play/pause |
| `Escape` | Deselect wallpaper |
| `Ctrl+R` | Reload configuration |
| `Ctrl+Q` | Quit GUI |
| `F5` | Refresh outputs |

## Troubleshooting

### "Daemon not running"

```bash
# Option 1: Use GUI button
# Click 🚀 Start Daemon

# Option 2: Start via systemd
systemctl --user start wayvid.service

# Option 3: Start manually
wayvid run &
```

### No monitors shown

- Ensure you're on a Wayland session
- Check compositor supports wlr-layer-shell
- Click refresh or restart GUI

### Wallpapers not appearing

- Verify file paths are accessible
- Supported formats: MP4, WebM, MKV, AVI, MOV
- For Workshop: Ensure Steam is installed

### GUI font issues (Chinese)

```bash
# Install CJK fonts
sudo pacman -S noto-fonts-cjk       # Arch
sudo apt install fonts-noto-cjk     # Debian/Ubuntu
```

### Workshop items not found

1. Verify Steam is installed
2. Check Wallpaper Engine is in library
3. Ensure items are downloaded
4. Try rescanning: Click refresh button

## Architecture

The GUI is built with:
- **egui/eframe**: Immediate mode GUI framework
- **rust-i18n**: Internationalization
- **IPC Client**: Unix socket communication

```
GUI ─────► IPC Socket ─────► wayvid daemon
            ▲                     │
            │                     ▼
         Response          Wayland compositor
```

## See Also

- [Configuration Guide](configuration.md)
- [CLI Control](../reference/cli.md)
- [IPC Protocol](../features/ipc.md)
- [Steam Workshop](../features/workshop.md)
