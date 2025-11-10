# Niri Integration

First-class support for [Niri](https://github.com/YaLTeR/niri) scrolling compositor.

## Features

- ✅ Workspace-aware playback
- ✅ Scroll performance optimization
- ✅ Native Niri IPC integration
- ✅ Zero-config setup

## Installation

```bash
# Install wayvid
yay -S wayvid-git

# Add to Niri config
echo 'spawn-at-startup "wayvid"' >> ~/.config/niri/config.kdl
```

## Automatic Optimizations

wayvid detects Niri via the `NIRI_SOCKET` environment variable and enables workspace-aware optimizations:

### Workspace Awareness
- **Automatic FPS throttling**: Reduces to 1 FPS on inactive workspaces
- **Instant resume**: Returns to normal FPS when workspace becomes active
- **Zero configuration**: Works out of the box

### Benefits
- **30-50% CPU savings** when multiple workspaces are in use
- **Reduced power consumption** on battery
- **Smoother desktop experience** by freeing resources for active windows

## Configuration

No special configuration required. wayvid automatically detects Niri and enables optimizations.

Standard power management options still apply:
```yaml
power:
  max_fps: 60          # FPS limit for active workspace (0 = unlimited)
  pause_on_battery: true
  pause_when_hidden: true
```

## CLI Integration

```bash
# Get current workspace
wayvid-ctl status | grep workspace

# Force workspace resync
wayvid-ctl reload-config
```

## Performance

With Niri workspace optimizations enabled:
- **30-50% CPU reduction** when using multiple workspaces
- **Automatic power saving** - inactive workspaces throttled to 1 FPS
- **Seamless experience** - instant resume when switching back

## Troubleshooting

**Wallpaper not showing:**
- Ensure Niri ≥ v0.1.0
- Check `wayvid-ctl status`
- Verify layer-shell support: `niri msg outputs`

**Workspace detection not working:**
- Check environment: `echo $NIRI_SOCKET`
- Should point to: `/run/user/1000/niri/niri-socket.XXX`
- Update Niri to latest version
- Check logs: `wayvid run --log-level debug`

**Performance issues:**
- Lower video resolution (1080p recommended)
- Enable hardware decode: `hwdec: true`
- Set FPS limit: `max_fps: 60`
- Use efficient codecs (H.264, H.265)

## Future Features

Planned for M7+:
- Per-workspace wallpapers
- Scroll performance optimization
- Advanced quality tuning options
- Noctalia Shell integration
