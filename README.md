# wayvid - Wayland Video Wallpaper Engine

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A dynamic video wallpaper engine for Wayland compositors, with priority support for **Hyprland** and **niri**. Compatible with Wallpaper Engine's video wallpaper parameters.

## Features

- ✅ **Native Wayland** - Uses `wlr-layer-shell` for background layer placement
- ✅ **Full Input Passthrough** - Wallpaper doesn't interfere with desktop interaction
- ✅ **Multi-Monitor** - Per-output configuration with hotplug support
- ✅ **Hardware Decode** - VA-API/NVDEC support with software fallback
- ✅ **Flexible Layouts** - Fill, Contain, Stretch, Cover, Centre modes
- ✅ **WE Compatible** - Supports core Wallpaper Engine video parameters
- ✅ **Low Resource** - Efficient playback with power-saving modes
- 🚧 **OpenGL Rendering** - (In progress - MVP uses simplified playback)

## Status: M1 MVP

**Current Release:** v0.1.0 (Milestone 1 - Single Output MVP)

This is an early MVP release. The core architecture is in place, but OpenGL rendering integration is simplified. See [Roadmap](#roadmap) for planned features.

### What Works
- ✅ Wayland layer-shell background surface creation
- ✅ libmpv integration with hardware decode
- ✅ Configuration system with per-output overrides
- ✅ Layout calculation (Fill/Contain/Stretch/Centre)
- ✅ CLI and capability checking

### What's Next (M2)
- 🔜 Full OpenGL/EGL rendering with mpv_render_context
- 🔜 Multi-output hotplug
- 🔜 Frame callbacks and vsync
- 🔜 Power management (pause on idle/battery)

## Supported Compositors

| Compositor | Status | Notes |
|------------|--------|-------|
| **Hyprland** | ✅ Primary | Tested on v0.35+ |
| **niri** | ✅ Primary | Tested on latest git |
| Sway | 🟡 Should work | Uses wlr-layer-shell |
| River | 🟡 Should work | Uses wlr-layer-shell |
| KDE/GNOME | ❌ Not supported | No wlr-layer-shell |

## Installation

### From Source (Recommended for Now)

#### Prerequisites

**System Dependencies:**
```bash
# Arch Linux
sudo pacman -S rust wayland libmpv mesa vulkan-icd-loader

# Ubuntu/Debian
sudo apt install rustc cargo libwayland-dev libmpv-dev libgl1-mesa-dev libegl1-mesa-dev

# Fedora
sudo dnf install rust cargo wayland-devel mpv-libs-devel mesa-libGL-devel mesa-libEGL-devel
```

**Hardware Decode (Optional but Recommended):**
```bash
# Intel
sudo pacman -S intel-media-driver libva-intel-driver  # Arch
sudo apt install intel-media-va-driver i965-va-driver # Ubuntu

# AMD
sudo pacman -S mesa-va-drivers  # Arch
sudo apt install mesa-va-drivers # Ubuntu

# NVIDIA
sudo pacman -S nvidia-utils nvidia-vaapi-driver  # Arch
sudo apt install nvidia-driver-XXX libnvidia-encode-XXX # Ubuntu (replace XXX with version)
```

#### Build and Install

```bash
# Clone repository
git clone https://github.com/yourusername/wayvid.git
cd wayvid

# Build release binary
cargo build --release

# Install to ~/.local/bin
install -Dm755 target/release/wayvid ~/.local/bin/wayvid

# Create config directory
mkdir -p ~/.config/wayvid

# Copy example config
cp configs/config.example.yaml ~/.config/wayvid/config.yaml

# Edit config with your video path
$EDITOR ~/.config/wayvid/config.yaml
```

### Package Managers (Coming Soon)

- 📦 **AUR**: `yay -S wayvid-git` (M2)
- ❄️ **Nix**: `nix run github:yourusername/wayvid` (M2)
- 📦 **AppImage**: Download from releases (M2)

## Configuration

### Basic Setup

Edit `~/.config/wayvid/config.yaml`:

```yaml
source:
  type: File
  path: "/home/user/Videos/wallpaper.mp4"

layout: Fill
loop: true
mute: true
hwdec: true
```

### Per-Monitor Configuration

```yaml
source:
  type: File
  path: "/home/user/Videos/default.mp4"

layout: Fill

per_output:
  # 4K monitor - use high-res video
  HDMI-A-1:
    source:
      type: File
      path: "/home/user/Videos/4k-wallpaper.mp4"
    layout: Fill
  
  # Laptop screen - use lower-power video
  eDP-1:
    source:
      type: File
      path: "/home/user/Videos/lowres.mp4"
    layout: Contain
    start_time: 10.0
```

### Layout Modes

- **Fill** (recommended): Scales video to fill screen, cropping excess
- **Contain**: Scales to fit inside screen, adds letterbox bars
- **Stretch**: Stretches to fill screen (distorts aspect ratio)
- **Centre**: No scaling, centers video at original size

### Power Saving

```yaml
power:
  pause_when_hidden: true  # Pause when monitor is off/disconnected
  pause_on_battery: false  # Pause on laptop battery
  max_fps: 30              # Limit to 30 FPS (0 = unlimited)
```

## Usage

### Run Wallpaper

```bash
# Use default config location
wayvid run

# Specify custom config
wayvid run --config /path/to/config.yaml

# With debug logging
wayvid run --log-level debug
```

### Check System Capabilities

```bash
wayvid check
```

Output example:
```
=== wayvid System Capability Check ===

[Wayland]
  ✓ WAYLAND_DISPLAY: wayland-1
  ✓ Connection: Established
  ✓ Protocols: Available
    - wl_compositor
    - wl_output
    - zwlr_layer_shell_v1
  ℹ Compositor: Hyprland

[Video Backend]
  ✓ Backend: libmpv
  ℹ mpv 0.37.0

[Hardware Decode]
  ✓ VA-API available
    Driver version: Intel iHD driver - 23.4.3
  ℹ VDPAU not available
```

### Autostart

#### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```conf
exec-once = wayvid run
```

#### niri

Add to `~/.config/niri/config.kdl`:

```kdl
spawn-at-startup "wayvid" "run"
```

#### systemd (Universal)

```bash
# Install service
mkdir -p ~/.config/systemd/user/
cp systemd/wayvid.service ~/.config/systemd/user/

# Enable and start
systemctl --user enable --now wayvid.service

# Check status
systemctl --user status wayvid

# View logs
journalctl --user -u wayvid -f
```

## Troubleshooting

### Black Screen / No Video

1. **Check output names:**
   ```bash
   wayvid check
   # Look for output names like HDMI-A-1, eDP-1
   ```

2. **Verify video file:**
   ```bash
   mpv /path/to/your/video.mp4  # Should play successfully
   ```

3. **Check logs:**
   ```bash
   wayvid run --log-level debug
   ```

4. **Disable hardware decode:**
   ```yaml
   hwdec: false
   ```

### Layer Not Behind Windows

- **Hyprland users**: Ensure you don't have conflicting layer rules
- **Verify layer-shell support:**
  ```bash
  wayvid check  # Should show zwlr_layer_shell_v1
  ```

### High CPU Usage

1. **Enable hardware decode** (if not already):
   ```yaml
   hwdec: true
   ```

2. **Check VA-API:**
   ```bash
   vainfo  # Should show available profiles
   ```

3. **Limit FPS:**
   ```yaml
   power:
     max_fps: 30
   ```

4. **Lower video resolution** or use more efficient codec (H.264 < H.265)

### Hardware Decode Not Working

```bash
# Check VA-API
vainfo

# NVIDIA users need nvidia-vaapi-driver
# Add to env if needed:
export LIBVA_DRIVER_NAME=nvidia

# Intel users on older hardware
export LIBVA_DRIVER_NAME=i965

# Check mpv's hwdec
mpv --hwdec=auto --log-file=mpv.log your-video.mp4
grep -i hwdec mpv.log
```

## Wallpaper Engine Compatibility

wayvid is compatible with core video wallpaper parameters:

| Parameter | Supported | Notes |
|-----------|-----------|-------|
| Video File | ✅ | MP4, WebM, MKV, AVI |
| Loop | ✅ | `loop: true/false` |
| Start Time | ✅ | `start_time: 10.5` |
| Playback Rate | ✅ | `playback_rate: 1.5` |
| Volume/Mute | ✅ | `mute: true`, `volume: 0.5` |
| Alignment | ✅ | Via `layout` modes |
| Scaling | ✅ | Via `layout` modes |

### Importing WE Wallpapers

**Manual Method (M1):**

1. Find the workshop folder:
   ```bash
   ls ~/.steam/steam/steamapps/workshop/content/431960/
   ```

2. Locate the video file (usually `.mp4` or `.webm`)

3. Point config to it:
   ```yaml
   source:
     type: File
     path: "/home/user/.steam/steam/steamapps/workshop/content/431960/123456789/scene.mp4"
   ```

**Automatic Import (Coming in M3):**
```bash
wayvid import-we /path/to/wallpaper_engine/project/
```

## Roadmap

### M1: Single Output MVP ✅ (Current)
- [x] Project structure and dependencies
- [x] Layer-shell background surface
- [x] libmpv integration (simplified)
- [x] Layout calculation
- [x] Configuration system
- [x] CLI and capability check
- [x] Documentation and examples

### M2: Multi-Output & Hotplug (3-5 weeks)
- [ ] Full OpenGL/EGL rendering pipeline
- [ ] mpv_render_context integration
- [ ] Frame callbacks and vsync
- [ ] Output hotplug detection
- [ ] Per-output player management
- [ ] Power saving implementation
- [ ] FPS/performance metrics

### M3: WE Import & Distribution (3-5 weeks)
- [ ] Wallpaper Engine project importer
- [ ] Flatpak package
- [ ] Debian/RPM packages
- [ ] Troubleshooting guide
- [ ] Performance profiling

### M4: Advanced Features (Ongoing)
- [ ] Shared decode optimization
- [ ] Static image fallback
- [ ] IPC/D-Bus control
- [ ] System tray (optional)
- [ ] Color management hints
- [ ] HDR planning

## Performance Notes

**Expected Resource Usage (1080p@30fps):**
- CPU: 2-5% (with hardware decode)
- GPU: 5-10% (depends on decode method)
- Memory: 100-300 MB per video

**4K Recommendations:**
- Hardware decode **required**
- Modern GPU (Intel 8th gen+, AMD RDNA+, NVIDIA GTX 1060+)
- For multiple 4K outputs, consider limiting FPS

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy
cargo fmt
```

### Features

```toml
[features]
default = ["video-mpv", "backend-wayland"]

video-mpv = ["dep:libmpv"]        # libmpv backend
video-gst = []                     # GStreamer (future)
backend-wayland = []               # Wayland support
config-toml = ["dep:toml"]         # TOML config
ipc = []                           # IPC control (future)
telemetry = []                     # Metrics (future)
```

### Architecture

```
wayvid/
├── src/
│   ├── main.rs              # Entry point, CLI
│   ├── config.rs            # Configuration loading
│   ├── core/
│   │   ├── layout.rs        # Layout calculation
│   │   └── types.rs         # Common types
│   ├── backend/
│   │   └── wayland/
│   │       ├── app.rs       # Event loop
│   │       ├── output.rs    # Output management
│   │       └── surface.rs   # Layer surface wrapper
│   ├── video/
│   │   └── mpv.rs           # libmpv player
│   └── ctl/
│       └── check.rs         # Capability checker
├── configs/                 # Example configs
├── systemd/                 # Service files
└── packaging/               # Package scripts (future)
```

## Contributing

Contributions are welcome! This is an early MVP, so there's plenty to improve.

**Priority areas:**
- OpenGL/EGL rendering implementation
- Testing on different compositors
- Performance optimization
- Documentation improvements

**Before contributing:**
1. Check existing issues
2. For major changes, open an issue first
3. Follow Rust style guide (`cargo fmt`)
4. Add tests where applicable

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- [mpv](https://mpv.io/) - Excellent media player library
- [Hyprland](https://hyprland.org/) - Modern Wayland compositor
- [niri](https://github.com/YaLTeR/niri) - Scrollable-tiling compositor
- [wlr-layer-shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) - Layer shell protocol
- [Wallpaper Engine](https://www.wallpaperengine.io/) - Inspiration

## FAQ

**Q: Why not just use mpv with --wid?**  
A: `--wid` doesn't work reliably with Wayland layer surfaces, and we need proper input passthrough and multi-output management.

**Q: Does this work on X11?**  
A: No, Wayland only. For X11, use `xwinwrap` or similar tools.

**Q: Can I use GIFs or images?**  
A: MVP only supports video. Static images planned for M4 (would integrate with existing wallpaper tools).

**Q: Why is CPU usage high?**  
A: Ensure hardware decode is enabled and working (`wayvid check`). Also check your video codec and resolution.

**Q: Can I control playback (pause/next/etc.)?**  
A: Not yet. IPC interface planned for M4.

**Q: Does this support interactive wallpapers?**  
A: No. Only video playback. No HTML/WebGL/scripts (different from WE's full feature set).

---

**Made with ❤️ for the Wayland community**

For issues, questions, or suggestions, please [open an issue](https://github.com/yourusername/wayvid/issues).
