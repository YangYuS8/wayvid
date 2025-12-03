# Configuration

Config file: `~/.config/wayvid/config.yaml`

## Basic Example

```yaml
source:
  type: file
  path: ~/Videos/wallpaper.mp4
layout: fill
volume: 0
loop: true
hwdec: true
```

## Source Types

### File
```yaml
source:
  type: file
  path: /path/to/video.mp4
```

### Directory (Playlist)
```yaml
source:
  type: directory
  path: ~/Videos/wallpapers/
  shuffle: true
  interval: 3600  # Change every hour
```

### Workshop
```yaml
source:
  type: workshop
  id: 1234567890
```

## Layout Modes

- **fill** - Cover screen, crop if needed (default)
- **contain** - Fit inside, may letterbox
- **stretch** - Fill exactly, may distort
- **centre** - Original size, centered

```yaml
layout: fill
```

## Per-Output Config

Different video per monitor:

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

## All Options

```yaml
# Video source
source:
  type: file|directory|workshop
  path: string
  id: number  # For workshop

# Playback
layout: fill|contain|stretch|centre
volume: 0-100
loop: true|false
start_time: 0.0
playback_rate: 1.0
mute: false

# Performance
hwdec: true

# HDR
hdr_mode: auto|force|disable
tone_mapping: auto

# Power
power:
  battery_threshold: 20
  pause_on_battery: false
```

## Reload Config

```bash
wayvid-ctl reload-config
```
