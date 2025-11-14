# Configuration

Config file: `~/.config/wayvid/config.yaml`

## Generate Default Config
```bash
wayvid --generate-config
```

## Basic Structure

```yaml
# Video source
source:
  type: file  # file, directory, workshop
  path: ~/Videos/wallpaper.mp4

# Playback settings
playback:
  loop: true
  volume: 0  # 0-100, 0 = muted
  paused: false

# Display configuration
outputs:
  - name: DP-1
    enabled: true
    mode: fit  # fit, fill, stretch
```

## Source Types

### Single File
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
  interval: 3600  # seconds
```

### Steam Workshop
```yaml
source:
  type: workshop
  id: 1234567890
```

## Layout Modes

Different scaling modes control how video fits the screen:

- **Fill** (default): Scale to cover entire screen, crop edges if needed
  - Maintains aspect ratio
  - No black bars, may crop video content
  - Uses MPV `panscan=1.0` for optimal filling
  
- **Contain**: Scale to fit inside screen, letterbox if needed
  - Maintains aspect ratio
  - May show black bars (letterbox/pillarbox)
  - Displays full video content
  
- **Stretch**: Stretch to fill screen, ignore aspect ratio
  - Distorts video to match screen dimensions
  - No black bars, no cropping
  - May look unnatural
  
- **Cover**: Alias for Fill mode
  
- **Centre**: Display at original size, centered
  - No scaling applied
  - Shows black bars if video smaller than screen

**Example:**
```yaml
layout: Fill  # or: Contain, Stretch, Cover, Centre
```

## Advanced Options

```yaml
# HDR settings
hdr:
  enabled: true
  target_nits: 1000

# Performance
performance:
  frame_skip: false
  decode_threads: auto

# Logging
log_level: info  # error, warn, info, debug
```

## Per-Output Configuration

```yaml
outputs:
  - name: DP-1
    source:
      type: file
      path: ~/Videos/left.mp4
  
  - name: HDMI-A-1
    source:
      type: file
      path: ~/Videos/right.mp4
```

## Reload Config
```bash
wayvid-ctl reload-config
```

See [Configuration Reference](../reference/config.md) for all options.
