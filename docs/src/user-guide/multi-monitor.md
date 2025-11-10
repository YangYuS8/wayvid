# Multi-Monitor Setup

Configure independent wallpapers per display.

## List Outputs
```bash
wayvid-ctl list-outputs
```

Output example:
```
DP-1: 2560x1440@144Hz (active)
HDMI-A-1: 1920x1080@60Hz (active)
```

## Independent Wallpapers

```yaml
outputs:
  - name: DP-1
    enabled: true
    source:
      type: file
      path: ~/Videos/main.mp4
  
  - name: HDMI-A-1
    enabled: true
    source:
      type: file
      path: ~/Videos/secondary.mp4
```

## Synchronized Playback

Use same source for all outputs:

```yaml
source:
  type: file
  path: ~/Videos/wallpaper.mp4

outputs:
  - name: DP-1
  - name: HDMI-A-1
```

## Disable Specific Output

```yaml
outputs:
  - name: DP-1
    enabled: true
  
  - name: HDMI-A-1
    enabled: false  # No wallpaper on this output
```

## Dynamic Output Changes

wayvid automatically handles:
- Hot-plug/unplug
- Resolution changes
- Refresh rate changes

No restart required.

## CLI Control

```bash
# Pause specific output
wayvid-ctl pause --output DP-1

# Resume all outputs
wayvid-ctl resume

# Stop output
wayvid-ctl stop --output HDMI-A-1
```

## Troubleshooting

**Output not detected:**
- Check `wayvid-ctl list-outputs`
- Verify output name matches exactly
- Restart compositor if needed

**Performance issues:**
- Reduce resolution
- Lower video quality
- Enable hardware decoding
- Use `--output` flag to test single display
