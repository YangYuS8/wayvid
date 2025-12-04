# CLI Commands

## wayvid-gui

Main GUI application (recommended).

```bash
wayvid-gui                    # Open GUI
wayvid-gui --minimized        # Start minimized to tray
wayvid-gui --version          # Show version
```

## wayvid-ctl

CLI control tool for scripting and automation.

### Basic Commands

```bash
wayvid-ctl status             # Show current status
wayvid-ctl status --json      # JSON output for scripts
wayvid-ctl outputs            # List available monitors
```

### Wallpaper Control

```bash
wayvid-ctl apply <path>       # Apply wallpaper
wayvid-ctl apply <path> --output DP-1  # Apply to specific monitor
wayvid-ctl pause              # Pause playback
wayvid-ctl resume             # Resume playback
wayvid-ctl stop               # Stop and clear wallpaper
```

### Examples

```bash
# Apply wallpaper to all monitors
wayvid-ctl apply ~/Videos/wallpaper.mp4

# Apply different wallpapers per monitor
wayvid-ctl apply ~/Videos/left.mp4 --output DP-1
wayvid-ctl apply ~/Videos/right.mp4 --output HDMI-A-1

# Check status in scripts
if wayvid-ctl status --json | jq -e '.playing'; then
  echo "Wallpaper is playing"
fi
```

## Legacy Commands

> **Note:** The standalone `wayvid` daemon binary has been removed in v0.5.
> Use `wayvid-gui` for the full experience, or `wayvid-ctl` for CLI control.

For Steam Workshop, use the GUI's Library tab or apply workshop videos directly:

```bash
wayvid-ctl apply ~/.steam/steam/steamapps/workshop/content/431960/<id>/video.mp4
```
