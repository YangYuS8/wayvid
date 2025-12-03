# Multi-Monitor

## List Outputs

```bash
wayvid-ctl list-outputs
# DP-1: 2560x1440@144Hz
# HDMI-A-1: 1920x1080@60Hz
```

## Different Video Per Monitor

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

```yaml
source:
  type: file
  path: ~/Videos/wallpaper.mp4
```

## Control Per Output

```bash
wayvid-ctl pause --output DP-1
wayvid-ctl resume --output DP-1
wayvid-ctl set-source ~/Videos/new.mp4 --output DP-1
```

## Hotplug

Monitors are detected automatically. No restart needed.
