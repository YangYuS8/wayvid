# CLI Commands

## wayvid-gui

Main GUI application with integrated playback engine.

```bash
wayvid-gui                    # Open GUI
wayvid-gui --minimized        # Start minimized to system tray
wayvid-gui --version          # Show version
wayvid-gui --help             # Show help
```

When running, wayvid-gui:
- Displays wallpapers via Wayland layer-shell
- Provides a graphical interface for browsing and applying wallpapers
- Starts an IPC server for wayvid-ctl communication
- Minimizes to system tray when closed

## wayvid-ctl

CLI control tool for scripting and automation. Communicates with wayvid-gui via IPC.

> **Note:** wayvid-ctl requires wayvid-gui to be running.

### Status Commands

```bash
wayvid-ctl status             # Show current status
wayvid-ctl status --json      # JSON output for scripts
wayvid-ctl outputs            # List available monitors
wayvid-ctl ping               # Check if daemon is running
```

Example output:
```
Wayvid Daemon Status
====================
Status:  Running
Version: 0.5.0-alpha.1

Active Outputs:
  eDP-1 [Playing]
    Wallpaper: /home/user/Videos/wallpaper.mp4
```

### Wallpaper Control

```bash
wayvid-ctl apply <path>                    # Apply to all monitors
wayvid-ctl apply <path> --output DP-1      # Apply to specific monitor
wayvid-ctl pause                           # Pause all playback
wayvid-ctl pause --output DP-1             # Pause specific monitor
wayvid-ctl resume                          # Resume all playback
wayvid-ctl stop                            # Stop and clear all wallpapers
wayvid-ctl stop --output DP-1              # Clear specific monitor
```

### Volume Control

```bash
wayvid-ctl volume 50 --output eDP-1        # Set volume to 50%
```

### Examples

```bash
# Apply wallpaper to all monitors
wayvid-ctl apply ~/Videos/wallpaper.mp4

# Apply different wallpapers per monitor
wayvid-ctl apply ~/Videos/left.mp4 --output DP-1
wayvid-ctl apply ~/Videos/right.mp4 --output HDMI-A-1

# Check status in scripts
if wayvid-ctl ping 2>/dev/null; then
  echo "wayvid is running"
else
  echo "wayvid is not running"
fi

# Get JSON status for scripting
wayvid-ctl status --json | jq '.outputs[].name'
```

### Steam Workshop

Apply Workshop wallpapers directly:

```bash
# Find workshop video
ls ~/.steam/steam/steamapps/workshop/content/431960/

# Apply workshop wallpaper
wayvid-ctl apply ~/.steam/steam/steamapps/workshop/content/431960/<id>/video.mp4
```

Or use the GUI's Library tab for a better browsing experience.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (connection failed, invalid arguments, etc.) |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Logging level (`error`, `warn`, `info`, `debug`, `trace`) |
| `XDG_RUNTIME_DIR` | IPC socket directory (default: `/run/user/$UID`) |
| `XDG_CONFIG_HOME` | Config directory (default: `~/.config`) |
| `XDG_CACHE_HOME` | Cache directory (default: `~/.cache`) |
