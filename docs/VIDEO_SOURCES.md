# Video Source Types

wayvid supports multiple video source types for maximum flexibility.

## Supported Source Types

### 1. Local Video File

Play a single video file from the filesystem.

```yaml
source:
  type: File
  path: ~/Videos/wallpaper.mp4
```

**Supported formats**: MP4, MKV, WebM, AVI, MOV, and any format supported by MPV.

### 2. HTTP/HTTPS URL Stream

Stream video directly from a URL.

```yaml
source:
  type: Url
  url: https://example.com/video.mp4
```

**Features**:
- Automatic caching (10 seconds buffer)
- Network error recovery
- Works with HTTP Live Streaming (HLS) and DASH

**Examples**:
```yaml
# Direct MP4 file
source:
  type: Url
  url: https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4

# HLS stream
source:
  type: Url
  url: https://example.com/stream/playlist.m3u8
```

### 3. RTSP Stream

Connect to RTSP streams (IP cameras, network video sources).

```yaml
source:
  type: Rtsp
  url: rtsp://192.168.1.100:554/stream
```

**Use cases**:
- IP security cameras
- Network video recorders
- Live streaming servers

**Example**:
```yaml
source:
  type: Rtsp
  url: rtsp://admin:password@192.168.1.50/live/main
```

### 4. Pipe Input

Read video from stdin or a named pipe.

```yaml
# From stdin
source:
  type: Pipe
  path: ""

# From named pipe
source:
  type: Pipe
  path: /tmp/video_pipe
```

**Use cases**:
- Dynamic video generation (e.g., with ffmpeg)
- Screen capture streaming
- Custom video processing pipelines

**Example**:
```bash
# Generate video with ffmpeg and pipe to wayvid
ffmpeg -f lavfi -i testsrc=duration=10:size=1920x1080:rate=30 -f matroska - | wayvid run

# Or use named pipe
mkfifo /tmp/video_pipe
ffmpeg -i input.mp4 -f matroska /tmp/video_pipe &
wayvid run --config pipe_config.yaml
```

### 5. Image Sequence / GIF

Display a sequence of images as animation.

```yaml
source:
  type: ImageSequence
  path: ~/Pictures/wallpapers/*.jpg
  fps: 30.0  # Optional, defaults to 30.0
```

**Features**:
- Supports wildcards (`*`, `?`)
- Automatic looping
- Configurable frame rate
- Works with: JPG, PNG, GIF, WebP, BMP

**Examples**:
```yaml
# Numbered sequence
source:
  type: ImageSequence
  path: ~/frames/frame_%04d.png
  fps: 24.0

# All images in directory
source:
  type: ImageSequence
  path: ~/Pictures/slideshow/*
  fps: 1.0  # 1 second per image

# Animated GIF
source:
  type: ImageSequence
  path: ~/Videos/animated.gif
```

### 6. Directory (Playlist)

⚠️ **Not yet implemented** - Planned for future release.

```yaml
source:
  type: Directory
  path: ~/Videos/wallpapers/
```

Will play all videos in the directory as a playlist.

## Configuration Examples

### Multi-Output with Different Sources

```yaml
# Default config
source:
  type: File
  path: ~/Videos/default.mp4
layout: Fill
loop: true

# Per-output overrides
per_output:
  HDMI-A-1:
    source:
      type: Url
      url: https://example.com/stream.mp4
  
  eDP-1:
    source:
      type: ImageSequence
      path: ~/Pictures/slideshow/*.jpg
      fps: 0.5  # 2 seconds per image
```

### Stream with Custom Settings

```yaml
source:
  type: Url
  url: https://example.com/video.m3u8
layout: Contain
loop: true
mute: false
volume: 0.3
playback_rate: 1.0
hwdec: true

# Power management
power:
  pause_on_battery: true
  max_fps: 30
```

### Pipe Input from Screen Capture

```yaml
source:
  type: Pipe
  path: ""  # stdin
layout: Stretch
mute: true
```

Then run:
```bash
wf-recorder -c h264 -f pipe:1 | wayvid run --config capture.yaml
```

## Performance Tips

### For Streaming Sources
- Enable hardware decoding: `hwdec: true`
- Set appropriate FPS limit: `power.max_fps: 30`
- Consider network bandwidth

### For Image Sequences
- Use compressed formats (JPG/WebP) for large collections
- Lower FPS for slideshows: `fps: 0.5`
- Hardware decoding is not needed

### For Pipes
- Use efficient encoding (h264, h265)
- Match resolution to output
- Consider using named pipes for complex pipelines

## Runtime Control

All sources can be switched dynamically without restarting:

```bash
# Switch to URL stream
wayvid-ctl switch --output eDP-1 https://example.com/new_video.mp4

# Switch to local file
wayvid-ctl switch --output eDP-1 ~/Videos/wallpaper.mp4

# Switch to RTSP stream
wayvid-ctl switch --output HDMI-A-1 rtsp://192.168.1.100:554/stream
```

Or modify the config file - changes are applied automatically with hot reload.

## Troubleshooting

### URL streams not playing
- Check network connectivity
- Verify URL is accessible: `curl -I <url>`
- Check firewall settings
- Some servers may block user agents - MPV handles this automatically

### RTSP streams failing
- Verify RTSP URL format: `rtsp://[user:pass@]host:port/path`
- Check if camera/server allows connections
- Try with VLC first to verify stream works

### Pipe input not working
- Ensure pipe provides valid video format (matroska recommended)
- Check pipe permissions
- Verify data is actually being written to pipe

### Image sequence not animating
- Check file path wildcards
- Verify FPS is set correctly
- Ensure `loop: true` is set in config

## See Also

- [Configuration Reference](CONFIG.md)
- [IPC Commands](IPC.md)
- [Power Management](POWER_MANAGEMENT.md)
