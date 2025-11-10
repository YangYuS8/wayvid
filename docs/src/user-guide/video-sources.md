# Video Sources

wayvid supports multiple video source types.

## File Source

Play a single video file.

```yaml
source:
  type: file
  path: /absolute/path/to/video.mp4
```

Supported formats: MP4, MKV, WEBM, MOV, AVI

## Directory Source

Play all videos in a directory.

```yaml
source:
  type: directory
  path: ~/Videos/wallpapers/
  shuffle: false
  interval: 1800  # Switch every 30 minutes
```

Options:
- `shuffle`: Randomize playback order
- `interval`: Seconds between videos (0 = play to end)

## Steam Workshop

Import Wallpaper Engine projects.

```yaml
source:
  type: workshop
  id: 1234567890  # Workshop item ID
```

### Find Workshop IDs

1. Subscribe to item in Steam Workshop
2. List available items:
```bash
wayvid workshop list
```

3. Get item details:
```bash
wayvid workshop info 1234567890
```

4. Import to config:
```bash
wayvid workshop import 1234567890
```

## URL Source (Planned)

Future support for streaming URLs.

## Supported Codecs

| Codec | Support |
|-------|---------|
| H.264 | ✅ Full |
| H.265/HEVC | ✅ Full |
| VP9 | ✅ Full |
| AV1 | ✅ Full |
| HDR10 | ✅ Full |
| Dolby Vision | ⚠️ Limited |

## Performance Tips

- Use H.264 for lower CPU usage
- Use HEVC/VP9 for better compression
- Avoid 4K on weak GPUs
- Pre-encode with target quality
