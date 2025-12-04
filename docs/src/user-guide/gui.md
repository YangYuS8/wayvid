# GUI

wayvid v0.5 introduces a GUI-first experience. Just run:

```bash
wayvid-gui
```

## Interface

The GUI has a sidebar navigation with four main views:

```
┌─────────────────────────────────────────────────┐
│  📁 Library    │                                │
│  📂 Folders    │   ┌─────┐ ┌─────┐ ┌─────┐    │
│  ⚙️ Settings   │   │ 🎬  │ │ 🎬  │ │ 🎬  │    │
│  ℹ️ About      │   │ vid │ │ vid │ │ vid │    │
│                │   └─────┘ └─────┘ └─────┘    │
│                │                                │
│                │   🔍 Search...                 │
├────────────────┴────────────────────────────────┤
│  Monitors: [DP-1 ✓] [HDMI-A-1] [eDP-1]         │
└─────────────────────────────────────────────────┘
```

## Views

### Library
Browse your wallpaper collection with thumbnails. Click to preview, double-click to apply.

### Folders
Manage source folders for wallpapers. Add/remove folders to scan.

### Settings
- **Autostart**: Launch wayvid-gui at login
- **Power management**: Pause on battery or fullscreen apps
- **Performance**: FPS limits, hardware decode options

### About
Version info and links.

## Usage

1. **Browse** - View wallpapers in Library tab
2. **Select monitor** - Click monitor in bottom bar
3. **Apply** - Double-click a wallpaper

## System Tray

The GUI minimizes to system tray when closed. Right-click tray icon for:
- Show/hide window
- Pause/resume playback
- Quit

## Autostart

Enable in Settings → Autostart, or manually:

```kdl
# niri: ~/.config/niri/config.kdl
spawn-at-startup "wayvid-gui" "--minimized"
```

```conf
# hyprland: ~/.config/hypr/hyprland.conf
exec-once = wayvid-gui --minimized
```

## Troubleshooting

**No monitors shown:**
- Check Wayland session is running
- Ensure compositor supports wlr-layer-shell

**Fonts broken (Chinese):**
```bash
sudo pacman -S noto-fonts-cjk  # Arch
```

**Thumbnails not loading:**
- Check `~/.cache/wayvid/thumbnails/` permissions
- Ensure ffmpeg is installed for video thumbnails
