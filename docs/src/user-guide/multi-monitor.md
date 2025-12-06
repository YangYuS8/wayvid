# Multi-Monitor

## Using GUI (Recommended)

1. Open `wayvid-gui`
2. Select monitor from the bottom bar
3. Browse wallpapers in Library tab
4. Double-click to apply to selected monitor

## List Outputs

```bash
wayvid-ctl outputs
# eDP-1: 1920x1080 @ (0, 0)
# DP-1: 2560x1440 @ (1920, 0)
```

## Different Video Per Monitor

### Via CLI

```bash
wayvid-ctl apply ~/Videos/left.mp4 --output DP-1
wayvid-ctl apply ~/Videos/right.mp4 --output HDMI-A-1
```

### Via Config File

```yaml
source:
  type: file
  path: ~/Videos/default.mp4

per_output:
  DP-1:
    source:
      type: file
      path: ~/Videos/left.mp4
  HDMI-A-1:
    source:
      type: file
      path: ~/Videos/right.mp4
```

## Same Video All Monitors

```bash
wayvid-ctl apply ~/Videos/wallpaper.mp4
```

Or via config:
```yaml
source:
  type: file
  path: ~/Videos/wallpaper.mp4
```

## Control Per Output

```bash
wayvid-ctl pause --output DP-1
wayvid-ctl resume --output DP-1
wayvid-ctl stop --output DP-1
wayvid-ctl apply ~/Videos/new.mp4 --output DP-1
```

## Hotplug

Monitors are detected automatically. No restart needed.
