# IPC Command Reference

Complete reference for wayvid's IPC system and the `wayvid-ctl` command-line client.

## Overview

wayvid provides runtime control through a Unix socket IPC (Inter-Process Communication) system. The `wayvid-ctl` command-line utility communicates with the running wayvid instance to control playback, change configuration, and query status.

### IPC Architecture

```
wayvid-ctl (Client) → Unix Socket → wayvid (Server)
                         ↓
                  JSON Request/Response
                         ↓
                  Command Processing
                         ↓
                  MPV/Surface Updates
```

**Socket Location:**
- Primary: `$XDG_RUNTIME_DIR/wayvid.sock`
- Fallback: `/tmp/wayvid-$USER.sock`

**Protocol:** JSON-based request/response

## Command Reference

### Playback Control

#### pause

Pause video playback on one or all outputs.

**Usage:**
```bash
wayvid-ctl pause [--output <name>]
```

**Options:**
- `--output <name>`: Target specific output (optional)

**Examples:**
```bash
# Pause all outputs
wayvid-ctl pause

# Pause specific output
wayvid-ctl pause --output eDP-1
```

**Response:**
```
Pause command sent successfully
```

---

#### resume

Resume video playback on one or all outputs.

**Usage:**
```bash
wayvid-ctl resume [--output <name>]
```

**Options:**
- `--output <name>`: Target specific output (optional)

**Examples:**
```bash
# Resume all outputs
wayvid-ctl resume

# Resume specific output
wayvid-ctl resume --output HDMI-A-1
```

**Response:**
```
Resume command sent successfully
```

---

#### seek

Seek to a specific time position.

**Usage:**
```bash
wayvid-ctl seek <time> --output <name>
```

**Arguments:**
- `<time>`: Time position in seconds (float)

**Options:**
- `--output <name>`: Target output (required)

**Examples:**
```bash
# Seek to 30 seconds
wayvid-ctl seek 30.0 --output eDP-1

# Seek to 2 minutes (120 seconds)
wayvid-ctl seek 120.5 --output HDMI-A-1
```

**Response:**
```
Seek command sent successfully
```

**Note:** Currently returns MPV error -4 in some cases. Functionality is being refined.

---

### Source Management

#### switch

Switch video source on a specific output.

**Usage:**
```bash
wayvid-ctl switch <source> --output <name>
```

**Arguments:**
- `<source>`: Path or URL to new video source

**Options:**
- `--output <name>`: Target output (required)

**Examples:**
```bash
# Switch to local file
wayvid-ctl switch ~/Videos/ocean.mp4 --output eDP-1

# Switch to URL
wayvid-ctl switch "https://example.com/video.mp4" --output HDMI-A-1

# Switch to RTSP stream
wayvid-ctl switch "rtsp://camera.local/stream" --output eDP-1
```

**Response:**
```
Switch command sent successfully
```

**Notes:**
- Path expansion (e.g., `~`) is supported
- URLs must be quoted if they contain special characters
- See [VIDEO_SOURCES.md](VIDEO_SOURCES.md) for supported source types

---

### Audio Control

#### volume

Set playback volume (0-100).

**Usage:**
```bash
wayvid-ctl volume <level> --output <name>
```

**Arguments:**
- `<level>`: Volume level (0.0 to 100.0)

**Options:**
- `--output <name>`: Target output (required)

**Examples:**
```bash
# Set volume to 50%
wayvid-ctl volume 50.0 --output eDP-1

# Set volume to 80%
wayvid-ctl volume 80.0 --output HDMI-A-1

# Mute (volume 0)
wayvid-ctl volume 0.0 --output eDP-1
```

**Response:**
```
Volume command sent successfully
```

**Note:** Currently returns MPV error -4 in some cases. Use config file + reload as workaround.

---

#### mute

Toggle audio mute on/off.

**Usage:**
```bash
wayvid-ctl mute --output <name>
```

**Options:**
- `--output <name>`: Target output (required)

**Examples:**
```bash
# Toggle mute on eDP-1
wayvid-ctl mute --output eDP-1

# Toggle mute on HDMI-A-1
wayvid-ctl mute --output HDMI-A-1
```

**Response:**
```
Mute command sent successfully
```

**Note:** Currently returns MPV error -4 in some cases. Functionality is being refined.

---

### Playback Speed

#### rate

Set playback speed/rate.

**Usage:**
```bash
wayvid-ctl rate <speed> --output <name>
```

**Arguments:**
- `<speed>`: Playback rate multiplier (e.g., 0.5 = half speed, 2.0 = double speed)

**Options:**
- `--output <name>`: Target output (required)

**Examples:**
```bash
# Normal speed
wayvid-ctl rate 1.0 --output eDP-1

# Half speed (slow motion)
wayvid-ctl rate 0.5 --output HDMI-A-1

# Double speed
wayvid-ctl rate 2.0 --output eDP-1

# 1.25x speed
wayvid-ctl rate 1.25 --output HDMI-A-1
```

**Response:**
```
Rate command sent successfully
```

**Typical Range:** 0.25 to 4.0
- Below 0.25: May cause audio issues
- Above 4.0: May cause frame drops

**Note:** Currently returns MPV error -4 in some cases. Use config file + reload as workaround.

---

### Layout Control

#### layout

Change video layout/scaling mode.

**Usage:**
```bash
wayvid-ctl layout <mode> --output <name>
```

**Arguments:**
- `<mode>`: Layout mode (see below)

**Options:**
- `--output <name>`: Target output (required)

**Layout Modes:**

| Mode | Aliases | Description |
|------|---------|-------------|
| `centre` | `center` | Center video, no scaling |
| `contain` | `fit` | Fit video inside output (preserve aspect) |
| `cover` | - | Fill output, crop if needed (preserve aspect) |
| `fill` | - | Fill output, stretch if needed (ignore aspect) |
| `stretch` | - | Stretch to fill (legacy alias for `fill`) |

**Examples:**
```bash
# Center video without scaling
wayvid-ctl layout centre --output eDP-1

# Fit video (letterbox/pillarbox)
wayvid-ctl layout contain --output HDMI-A-1

# Fill output (crop to fill)
wayvid-ctl layout cover --output eDP-1

# Stretch to fill (ignore aspect ratio)
wayvid-ctl layout fill --output HDMI-A-1
```

**Response:**
```
Layout command sent successfully
```

**Visual Guide:**

```
centre:   [  📺  ]  (small, centered)
contain:  [📺📺📺]  (fit, bars if needed)
cover:    [📺📺📺]  (fill, crop if needed)
fill:     [📺📺📺]  (stretch to fill)
```

---

### System Control

#### status

Display status information for all outputs.

**Usage:**
```bash
wayvid-ctl status
```

**Examples:**
```bash
wayvid-ctl status
```

**Response:**
```
Status command sent successfully
Check wayvid console output for status information
```

**Console Output (wayvid):**
```
=== wayvid Status ===
Outputs: 2
  - eDP-1: 1920x1080, video: ~/Videos/wallpaper.mp4
  - HDMI-A-1: 2560x1440, video: ~/Videos/ocean.mp4
```

**Note:** Status information is currently logged to wayvid's console. Future versions will return structured data to the client.

---

#### reload

Reload configuration from file.

**Usage:**
```bash
wayvid-ctl reload
```

**Examples:**
```bash
wayvid-ctl reload
```

**Response:**
```
Reload command sent successfully
```

**Effects:**
- Reloads `~/.config/wayvid/config.yaml`
- Updates layout, volume, playback rate
- Switches sources if changed in config
- Applies per-output overrides

**Use Cases:**
- Manual config reload (file watcher is automatic)
- Test configuration changes
- Force refresh

**Note:** Configuration hot reload happens automatically when you save the config file. Use this command to force a reload if needed.

---

#### quit

Shut down wayvid gracefully.

**Usage:**
```bash
wayvid-ctl quit
```

**Examples:**
```bash
wayvid-ctl quit
```

**Response:**
```
Quit command sent successfully
```

**Effects:**
- Stops all video playback
- Closes all surfaces
- Cleans up IPC socket
- Exits wayvid process

**Note:** Graceful shutdown ensures resources are properly released.

---

## Common Workflows

### Pause and Resume

```bash
# Pause everything
wayvid-ctl pause

# Take a break...

# Resume everything
wayvid-ctl resume
```

### Per-Monitor Control

```bash
# Pause laptop screen, keep external playing
wayvid-ctl pause --output eDP-1

# Resume laptop, pause external
wayvid-ctl resume --output eDP-1
wayvid-ctl pause --output HDMI-A-1
```

### Change Video on the Fly

```bash
# Switch video on main monitor
wayvid-ctl switch ~/Videos/new-wallpaper.mp4 --output HDMI-A-1

# Or switch to a stream
wayvid-ctl switch "https://example.com/stream.mp4" --output eDP-1
```

### Adjust Layout

```bash
# Try different layouts to find what looks best
wayvid-ctl layout contain --output eDP-1
wayvid-ctl layout cover --output eDP-1
wayvid-ctl layout fill --output eDP-1
```

### Test Configuration Changes

```bash
# Edit config
vim ~/.config/wayvid/config.yaml

# Force reload (though it should auto-reload)
wayvid-ctl reload

# Check status
wayvid-ctl status
```

### Stream Management

```bash
# Start stream
wayvid-ctl switch "rtsp://camera.local/stream" --output HDMI-A-1

# Pause stream (keeps buffer)
wayvid-ctl pause --output HDMI-A-1

# Resume stream
wayvid-ctl resume --output HDMI-A-1

# Switch back to file
wayvid-ctl switch ~/Videos/wallpaper.mp4 --output HDMI-A-1
```

## Scripting

### Check if wayvid is Running

```bash
#!/bin/bash
if [ -S "$XDG_RUNTIME_DIR/wayvid.sock" ]; then
    echo "wayvid is running"
else
    echo "wayvid is not running"
fi
```

### Toggle Pause/Resume

```bash
#!/bin/bash
# Note: This requires tracking state externally or using a file lock
if [ -f /tmp/wayvid-paused ]; then
    wayvid-ctl resume
    rm /tmp/wayvid-paused
else
    wayvid-ctl pause
    touch /tmp/wayvid-paused
fi
```

### Cycle Through Videos

```bash
#!/bin/bash
VIDEOS=(
    ~/Videos/ocean.mp4
    ~/Videos/forest.mp4
    ~/Videos/space.mp4
)

INDEX_FILE=/tmp/wayvid-index
if [ ! -f "$INDEX_FILE" ]; then
    echo 0 > "$INDEX_FILE"
fi

INDEX=$(cat "$INDEX_FILE")
VIDEO="${VIDEOS[$INDEX]}"

wayvid-ctl switch "$VIDEO" --output eDP-1

NEXT_INDEX=$(( (INDEX + 1) % ${#VIDEOS[@]} ))
echo "$NEXT_INDEX" > "$INDEX_FILE"
```

### Keyboard Shortcuts (Hyprland)

Add to `~/.config/hypr/hyprland.conf`:

```conf
# Pause/resume wallpaper
bind = SUPER, P, exec, wayvid-ctl pause
bind = SUPER SHIFT, P, exec, wayvid-ctl resume

# Switch wallpaper
bind = SUPER, W, exec, wayvid-ctl switch ~/Videos/next.mp4 --output eDP-1

# Toggle layout
bind = SUPER, L, exec, wayvid-ctl layout contain --output eDP-1
```

## Error Handling

### Command Failed

If a command fails, check:

1. **Is wayvid running?**
   ```bash
   ps aux | grep wayvid
   ```

2. **Is the socket present?**
   ```bash
   ls -l $XDG_RUNTIME_DIR/wayvid.sock
   ```

3. **Check wayvid logs:**
   ```bash
   # If running from terminal, check output
   # If running as service, check journal:
   journalctl --user -u wayvid -f
   ```

### Socket Not Found

**Error:**
```
Error: No such file or directory (os error 2)
```

**Solution:**
- Ensure wayvid is running: `ps aux | grep wayvid`
- Start wayvid: `wayvid` or restart via compositor/systemd
- Check socket location: `echo $XDG_RUNTIME_DIR`

### Permission Denied

**Error:**
```
Error: Permission denied (os error 13)
```

**Solution:**
- Check socket permissions: `ls -l $XDG_RUNTIME_DIR/wayvid.sock`
- Ensure you're running as the same user that started wayvid
- Kill stale socket: `rm $XDG_RUNTIME_DIR/wayvid.sock`

### Output Not Found

**Error:**
```
Output not found: HDMI-A-1
```

**Solution:**
- List available outputs: `wayvid-ctl status`
- Check output names in compositor settings
- Ensure output is connected and active

### MPV Error -4

**Error:**
```
MPV error -4 (property unavailable)
```

**Affected Commands:** seek, volume, mute

**Workaround:**
1. Use config file + hot reload:
   ```bash
   vim ~/.config/wayvid/config.yaml  # Change volume/rate
   wayvid-ctl reload
   ```

2. Restart with new settings:
   ```bash
   wayvid-ctl quit
   wayvid  # With updated config
   ```

**Status:** Issue is being investigated for M4.

## IPC Protocol (Advanced)

For developers integrating with wayvid, the raw IPC protocol is JSON-based.

### Request Format

```json
{
  "command": "CommandName",
  "params": {
    "field1": "value1",
    "field2": 123
  }
}
```

### Response Format

**Success:**
```json
{
  "success": true,
  "data": null
}
```

**Error:**
```json
{
  "success": false,
  "error": "Error message"
}
```

### Command Examples

**Pause (all):**
```json
{"command": "Pause", "params": {"output": null}}
```

**Pause (specific):**
```json
{"command": "Pause", "params": {"output": "eDP-1"}}
```

**Switch source:**
```json
{
  "command": "SwitchSource",
  "params": {
    "output": "HDMI-A-1",
    "source": "/home/user/video.mp4"
  }
}
```

**Set layout:**
```json
{
  "command": "SetLayout",
  "params": {
    "output": "eDP-1",
    "layout": "contain"
  }
}
```

### Socket Communication

```bash
# Send command directly (advanced)
echo '{"command":"GetStatus","params":{}}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/wayvid.sock
```

## Troubleshooting

### Commands Seem Slow

**Check:**
- MPV is using hardware decode: Check wayvid output for `[hwdec]` messages
- CPU usage: `htop` or `top`
- Compositor performance: Try with simpler wallpaper

**Fix:**
- Enable hardware decoding in config
- Lower video resolution/bitrate
- Use `contain` layout instead of `cover` (less scaling)

### Source Switch Fails

**Check:**
- File exists: `ls -l <path>`
- File is readable: `file <path>`
- URL is accessible: `curl -I <url>`
- MPV can play it: `mpv <path>`

**Fix:**
- Use absolute paths
- Check file permissions
- Test URL in browser
- Try simpler video format (H.264 MP4)

### Layout Changes Not Visible

**Check:**
- Video aspect ratio matches output
- Using correct output name
- Compositor scaling settings

**Fix:**
- Try different layouts: `contain`, `cover`, `fill`
- Check `wayvid-ctl status` for current layout
- Restart wayvid if stuck

### Reload Not Working

**Check:**
- Config file syntax: `yamllint ~/.config/wayvid/config.yaml`
- Config file modified time: `ls -l ~/.config/wayvid/config.yaml`
- File watcher messages in wayvid output

**Fix:**
- Validate YAML syntax
- Use `wayvid-ctl reload` to force reload
- Check wayvid logs for reload errors

## See Also

- [Quick Start Guide](QUICKSTART.md) - Installation and setup
- [Video Sources Guide](VIDEO_SOURCES.md) - Source types and configuration
- [README](../README.md) - Project overview and features

---

**Last Updated:** October 23, 2025  
**Version:** v0.3.0
