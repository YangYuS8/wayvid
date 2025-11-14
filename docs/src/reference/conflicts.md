# Wallpaper Manager Conflicts

## Problem Description

wayvid uses the Wayland Layer Shell protocol's `Background` layer to display wallpapers. If other wallpaper managers (such as swww, hyprpaper, swaybg, etc.) are running simultaneously on your system, they may cover wayvid's output, making the video wallpaper invisible.

## Common Conflicting Wallpaper Managers

### swww

**Symptoms**: Desktop still shows static wallpaper after starting wayvid

**Detection**:
```bash
ps aux | grep swww-daemon
```

**Solution**:
```bash
# Temporary stop
killall swww-daemon

# Permanent disable: Remove swww-related commands from compositor config
# Example for Niri: ~/.config/niri/config.kdl
# Comment out or delete these lines:
# spawn-sh-at-startup "swww-daemon"
# spawn-sh-at-startup "swww img /path/to/image.png"
```

### hyprpaper (Hyprland)

**Detection**:
```bash
ps aux | grep hyprpaper
```

**Solution**:
```bash
# Stop hyprpaper
killall hyprpaper

# Remove from ~/.config/hypr/hyprland.conf:
# exec-once = hyprpaper
```

### swaybg (Sway/SwayFX)

**Detection**:
```bash
ps aux | grep swaybg
```

**Solution**:
```bash
# Stop swaybg
killall swaybg

# Remove from Sway config:
# exec swaybg -i /path/to/image.png
```

## Automatic Detection

wayvid automatically detects these conflicting programs at startup and displays warnings in the logs:

```
⚠️  Detected swww-daemon running
⚠️  swww and wayvid both use the Background layer
⚠️  This may cause wayvid to be hidden behind swww
⚠️  To fix: run 'killall swww-daemon' before starting wayvid
```

## Recommended Configuration Steps

1. **Stop existing wallpaper managers**:
   ```bash
   killall swww-daemon hyprpaper swaybg 2>/dev/null
   ```

2. **Remove their autostart from compositor configuration**

3. **Add wayvid to autostart**:
   ```bash
   # Niri example
   spawn-at-startup "wayvid" "run"
   
   # Hyprland example
   exec-once = wayvid run
   
   # Sway example
   exec wayvid run
   ```

4. **Restart compositor or manually start wayvid**

## Verification

Check running processes after startup:
```bash
ps aux | grep -E "(wayvid|swww|hyprpaper|swaybg)" | grep -v grep
```

You should only see `wayvid` running.

## Technical Details

All these wallpaper managers use the Wayland Layer Shell protocol's `Background` layer. This layer is the bottom-most layer beneath all windows, and multiple programs cannot coexist on the same layer. The last program started typically covers previous ones.

Future versions may:
- Provide configuration options to choose different layers (e.g., `Bottom`)
- Automatically detect and attempt to terminate conflicting processes
- Deeper integration with compositors
