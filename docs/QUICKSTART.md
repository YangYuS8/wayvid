# Quick Start Guide

Get wayvid up and running in 5 minutes.

## Prerequisites

- **Wayland compositor** with wlr-layer-shell support (Hyprland, niri, Sway, River)
- **Video file** (MP4, WebM, MKV, etc.) or Wallpaper Engine project

## Installation

### Option 1: AppImage (Easiest - Any Distribution)

```bash
# Download
wget https://github.com/YangYuS8/wayvid/releases/download/v0.3.0/wayvid-0.3.0-x86_64.AppImage
chmod +x wayvid-0.3.0-x86_64.AppImage

# Move to PATH (optional)
mv wayvid-0.3.0-x86_64.AppImage ~/.local/bin/wayvid
```

### Option 2: AUR (Arch Linux)

```bash
yay -S wayvid-git
```

### Option 3: Nix Flakes

```bash
nix profile install github:YangYuS8/wayvid
```

### Option 4: From Source

```bash
# Install dependencies
# Arch: sudo pacman -S rust wayland libmpv mesa
# Ubuntu: sudo apt install rustc cargo libwayland-dev libmpv-dev libgl1-mesa-dev libegl1-mesa-dev
# Fedora: sudo dnf install rust cargo wayland-devel mpv-libs-devel mesa-libGL-devel

git clone https://github.com/YangYuS8/wayvid.git
cd wayvid
cargo build --release --all-features
sudo install -Dm755 target/release/wayvid /usr/local/bin/wayvid
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/wayvid-ctl
```

## Configuration

### Option A: Import from Wallpaper Engine (Easiest)

If you have Wallpaper Engine projects:

```bash
# Import WE project
wayvid import ~/.steam/steam/steamapps/workshop/content/431960/YOUR_WORKSHOP_ID \
  --output ~/.config/wayvid/config.yaml

# Or from local WE project
wayvid import ~/path/to/we-project --output ~/.config/wayvid/config.yaml
```

This automatically converts all WE settings (video path, playback rate, alignment, volume, etc.) to wayvid config!

### Option B: Create Configuration Manually

```bash
mkdir -p ~/.config/wayvid
cat > ~/.config/wayvid/config.yaml << 'EOF'
source:
  type: File
  path: "~/Videos/wallpaper.mp4"  # Change this to your video

layout: Fill
loop: true
mute: true
hwdec: true

power:
  max_fps: 30
EOF
```

## Test Run

```bash
# Check system capabilities
wayvid check

# Run wayvid (Ctrl+C to stop)
wayvid run

# Or with custom config
wayvid run --config ~/my-config.yaml

# If working, set up autostart (see below)
```

### Runtime Control

Control playback without restarting:

```bash
# Pause/Resume
wayvid-ctl pause
wayvid-ctl resume

# Switch video
wayvid-ctl switch ~/Videos/another-video.mp4

# Adjust playback
wayvid-ctl rate 1.5      # 1.5x speed
wayvid-ctl volume 0.5    # 50% volume
wayvid-ctl seek 30.0     # Jump to 30 seconds

# Check status
wayvid-ctl status
```

## Autostart

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```conf
exec-once = wayvid run
```

### niri

Add to `~/.config/niri/config.kdl`:

```kdl
spawn-at-startup "wayvid" "run"
```

### Sway/River

Add to your compositor config:

```bash
exec wayvid run
```

### systemd (Any compositor)

```bash
# Create service file
cat > ~/.config/systemd/user/wayvid.service << 'EOF'
[Unit]
Description=wayvid - Wayland Video Wallpaper Engine
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/wayvid run
Restart=on-failure
RestartSec=5

[Install]
WantedBy=graphical-session.target
EOF

# Enable and start
systemctl --user daemon-reload
systemctl --user enable --now wayvid.service

# Check status
systemctl --user status wayvid
```

## Common Tasks

### Change Video

Edit `~/.config/wayvid/config.yaml` - changes apply automatically!

### Control Playback

```bash
wayvid-ctl pause              # Pause all outputs
wayvid-ctl resume             # Resume playback
wayvid-ctl status             # Show current status
```

### Switch Video Live

```bash
wayvid-ctl switch --output eDP-1 ~/Videos/new_video.mp4
```

### Different Video Per Monitor

```yaml
# In config.yaml
source:
  type: File
  path: "~/Videos/default.mp4"

per_output:
  HDMI-A-1:
    source:
      type: File
      path: "~/Videos/4k_video.mp4"
  
  eDP-1:
    source:
      type: File
      path: "~/Videos/laptop_video.mp4"
```

Get output names with: `wayvid check`

## Troubleshooting

### Black Screen

1. Verify video works: `mpv ~/Videos/wallpaper.mp4`
2. Check output names: `wayvid check`
3. Try without hwdec: Set `hwdec: false` in config
4. Check logs: `wayvid run --log-level debug`

### High CPU Usage

1. Enable hardware decode: `hwdec: true`
2. Verify VA-API: `vainfo`
3. Limit FPS: Set `power.max_fps: 30`
4. Use lower resolution video

### Not Starting Automatically

1. Check service status: `systemctl --user status wayvid`
2. View logs: `journalctl --user -u wayvid -f`
3. Test manual start: `wayvid run`

## Next Steps

- **Runtime Control**: See [IPC Documentation](../docs/IPC.md)
- **Advanced Sources**: See [Video Sources Guide](../docs/VIDEO_SOURCES.md)
- **Power Management**: Adjust `power` settings in config
- **Multiple Monitors**: Use `per_output` overrides

## Getting Help

- Check [README](../README.md) for detailed documentation
- View [example configs](../configs/)
- Open an issue on GitHub
- Check existing issues for solutions

## Quick Reference

```bash
# Main commands
wayvid run                              # Start wallpaper
wayvid check                            # Check system
wayvid run --config custom.yaml         # Custom config

# Control commands
wayvid-ctl pause                        # Pause
wayvid-ctl resume                       # Resume
wayvid-ctl status                       # Status
wayvid-ctl reload                       # Reload config
wayvid-ctl quit                         # Stop

# Service commands
systemctl --user start wayvid           # Start service
systemctl --user stop wayvid            # Stop service
systemctl --user restart wayvid         # Restart
systemctl --user status wayvid          # Check status
journalctl --user -u wayvid -f          # View logs
```

---

**Need more help?** Check the [main README](../README.md) or [open an issue](https://github.com/yourusername/wayvid/issues).
