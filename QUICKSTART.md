# Quick Start Guide

Get wayvid running in 5 minutes!

## Prerequisites

Ensure you're running:
- ✅ **Hyprland** or **niri** compositor (or other wlr-based compositor)
- ✅ **Hardware video decode** drivers installed (recommended)

## Installation

### 1. Install Dependencies

**Arch Linux:**
```bash
sudo pacman -S rust wayland libmpv mesa intel-media-driver  # or amd/nvidia drivers
```

**Ubuntu/Debian:**
```bash
sudo apt install rustc cargo libwayland-dev libmpv-dev libgl1-mesa-dev intel-media-va-driver
```

**Fedora:**
```bash
sudo dnf install rust cargo wayland-devel mpv-libs-devel mesa-libGL-devel mesa-va-drivers
```

### 2. Build wayvid

```bash
git clone https://github.com/yourusername/wayvid.git
cd wayvid
cargo build --release
sudo install -Dm755 target/release/wayvid /usr/local/bin/wayvid
```

### 3. Configure

```bash
mkdir -p ~/.config/wayvid
cp configs/config.example.yaml ~/.config/wayvid/config.yaml
```

Edit the config and set your video path:
```yaml
source:
  type: File
  path: "/home/yourusername/Videos/your-video.mp4"  # ← Change this!

layout: Fill
loop: true
mute: true
hwdec: true
```

### 4. Test

```bash
# Check system compatibility
wayvid check

# Run wallpaper
wayvid run
```

You should now see your video as the wallpaper!

## Autostart

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:
```conf
exec-once = wayvid run
```

### niri

Add to your niri config:
```kdl
spawn-at-startup "wayvid" "run"
```

### systemd (Any compositor)

```bash
# Install service
mkdir -p ~/.config/systemd/user/
cp systemd/wayvid.service ~/.config/systemd/user/

# Enable
systemctl --user enable --now wayvid.service
```

## Common Issues

### "No video showing"

1. Check your video file works:
   ```bash
   mpv /path/to/your/video.mp4
   ```

2. Check output names match your config:
   ```bash
   wayvid check
   ```

3. Try with debug logging:
   ```bash
   wayvid run --log-level debug
   ```

### "High CPU usage"

1. Verify hardware decode is working:
   ```bash
   vainfo  # Should show driver info
   ```

2. Enable hwdec in config:
   ```yaml
   hwdec: true
   ```

3. Consider limiting FPS:
   ```yaml
   power:
     max_fps: 30
   ```

### "Wallpaper above windows"

- This is a compositor layer configuration issue
- For Hyprland: Check you don't have conflicting layer rules
- Try restarting the compositor

## Next Steps

- [Full Documentation](README.md)
- [Configuration Reference](configs/config.example.yaml)
- [Troubleshooting Guide](README.md#troubleshooting)
- [Report Issues](https://github.com/yourusername/wayvid/issues)

## Example Videos

Free video sources for testing:
- https://pixabay.com/videos/
- https://www.pexels.com/videos/
- https://coverr.co/

Recommended formats: MP4 (H.264), WebM (VP9), up to 4K

---

**Need help?** Open an issue or check the full README!
