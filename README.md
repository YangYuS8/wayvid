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
- ✅ **Multi-Source Support** - Local files, HTTP/RTSP streams, pipes, image sequences
- ✅ **Runtime Control** - Control playback via `wayvid-ctl` (pause, seek, switch, etc.)
- ✅ **Hot Reload** - Config changes applied instantly without restart
- ✅ **Power Management** - Battery detection, FPS limiting, auto-pause
- ✅ **OpenGL Rendering** - Full EGL/OpenGL integration with mpv render API
- ✅ **Low Resource** - Efficient playback with intelligent caching
- ✅ **HDR Support** - Automatic HDR10/HLG detection with tone mapping

## HDR Support 🎨

wayvid now supports **HDR (High Dynamic Range)** content including HDR10 and HLG formats. The player automatically detects HDR content and applies intelligent tone mapping to display it correctly on SDR displays.

### Features

- 🎯 **Automatic Detection**: HDR10, HLG, and Dolby Vision content recognition
- 🎨 **Smart Tone Mapping**: High-quality HDR to SDR conversion
- 🔧 **Content-Aware**: Automatic parameter optimization based on content type
- ⚡ **Multiple Algorithms**: 5 tone mapping algorithms (Hable, Mobius, Reinhard, BT.2390, Clip)
- 📊 **Performance Modes**: Balance quality and GPU load

### Quick Start

```yaml
# Enable automatic HDR handling (default)
hdr_mode: auto

# Configure tone mapping
tone_mapping:
  algorithm: hable      # Best overall quality
  param: 1.0           # Auto-optimized for content
  compute_peak: true   # Dynamic peak detection
  mode: hybrid         # Balanced processing
```

### Tone Mapping Algorithms

| Algorithm | Best For | Quality | Performance |
|-----------|----------|---------|-------------|
| **hable** ⭐ | Movies, general | Excellent | Moderate |
| **mobius** | Animation, bright | Excellent details | Good |
| **reinhard** | Low-end hardware | Good | Fast |
| **bt2390** | Professional | Reference | Good |
| **clip** | Testing | Poor | Fastest |

### Content-Aware Optimization

wayvid automatically adjusts tone mapping based on detected content:

- **Cinema** (peak >2000 nits): Higher contrast, RGB mode
- **Animation**: Detail preservation, luma mode
- **Documentary**: Natural ITU standard
- **Low DR**: Gentle mapping

### Example Configurations

```yaml
# Cinema/Movie (optimized)
hdr_mode: auto
tone_mapping:
  algorithm: hable
  param: 1.2       # Higher contrast
  mode: rgb        # Better cinema look

# Animation (vibrant colors)
hdr_mode: auto
tone_mapping:
  algorithm: mobius
  param: 0.35      # Preserve details
  mode: luma       # Keep colors saturated

# Performance mode
hdr_mode: auto
tone_mapping:
  algorithm: reinhard
  compute_peak: false  # Faster
  mode: luma
```

### Documentation

For comprehensive HDR configuration guide, see:
- 📖 [HDR User Guide](docs/HDR_USER_GUIDE.md) - Complete setup and tuning
- 📝 [HDR Examples](examples/hdr-config.yaml) - 8+ configuration examples
- 🧪 [Test Script](scripts/test-hdr-tonemapping.sh) - Algorithm comparison

### Requirements

- MPV >= 0.35 (for full HDR support)
- GPU with OpenGL 3.3+ support
- HDR video content (HDR10, HLG)

## Status: M5 Complete, M6 In Progress

**Current Release:** v0.3.0 (Milestone 5 - Performance & Polish)  
**Next Milestone:** v0.5.0 (M6 - Niri + Workshop Integration)

wayvid is now **production-ready** with comprehensive performance optimizations and HDR support. We're actively developing **Steam Workshop integration** and **Niri-specific optimizations** to become the standard wallpaper solution for the Niri ecosystem.

### What Works
- ✅ Wayland layer-shell background surface creation
- ✅ Full OpenGL/EGL rendering with mpv_render_context
- ✅ Multi-output with hotplug support
- ✅ Frame callbacks and vsync
- ✅ Power management (battery, FPS limit, pause-on-hidden)
- ✅ **Runtime Control** - IPC via wayvid-ctl
- ✅ **Configuration Hot Reload** - No restart needed
- ✅ **Multi-Source Support** - Files, URLs, RTSP, Pipes, Images
- ✅ **Wallpaper Engine Import** - Direct WE project conversion
- ✅ Layout calculation (Fill/Contain/Stretch/Cover/Centre)
- ✅ Per-output configuration overrides
- ✅ Hardware decode with VA-API/NVDEC
- ✅ **HDR Support** - HDR10/HLG detection with smart tone mapping
- ✅ **Shared Decode** - 60% CPU reduction for multi-display setups
- ✅ **Memory Optimization** - Intelligent buffer pooling
- ✅ **Frame Skip Intelligence** - Adaptive performance under load

### Distribution Support
- ✅ **AppImage** - Universal Linux binary
- ✅ **AUR** - Arch Linux packages (git + stable)
- ✅ **Nix Flakes** - NixOS and Home Manager integration
- ✅ **Source Build** - All major distributions

### What's Next (M6) 🚀
- 🚧 **Steam Workshop Integration** - One-click import from WE Workshop ([#23](https://github.com/YangYuS8/wayvid/issues/23))
- � **Niri Optimizations** - Workspace-aware, scroll-optimized ([#24](https://github.com/YangYuS8/wayvid/issues/24))
- 🔜 **Playlist Support** - Directory sources with rotation ([#3](https://github.com/YangYuS8/wayvid/issues/3))
- 🔜 **Arch Linux Enhancements** - Improved AUR packaging ([#25](https://github.com/YangYuS8/wayvid/issues/25))
- 🔜 **Noctalia Shell Preparation** - D-Bus interface, theme integration

## Supported Compositors

| Compositor | Status | Notes |
|------------|--------|-------|
| **Hyprland** | ✅ Primary | Tested on v0.35+ |
| **niri** | ✅ Primary | Tested on latest git |
| Sway | 🟡 Should work | Uses wlr-layer-shell |
| River | 🟡 Should work | Uses wlr-layer-shell |
| KDE/GNOME | ❌ Not supported | No wlr-layer-shell |

## Installation

### AppImage (Universal Linux - Recommended)

Download the latest AppImage from [Releases](https://github.com/YangYuS8/wayvid/releases):

```bash
# Download
wget https://github.com/YangYuS8/wayvid/releases/download/v0.3.0/wayvid-0.3.0-x86_64.AppImage

# Make executable
chmod +x wayvid-0.3.0-x86_64.AppImage

# Run directly
./wayvid-0.3.0-x86_64.AppImage --version

# Optional: Move to PATH
mv wayvid-0.3.0-x86_64.AppImage ~/.local/bin/wayvid
```

**Features**:
- ✅ Works on any Linux distribution (Ubuntu, Fedora, Arch, Debian, etc.)
- ✅ No installation required
- ✅ Includes both `wayvid` and `wayvid-ctl`
- ✅ Self-contained with all dependencies

**Usage**:
```bash
# Run wayvid
wayvid run --config ~/.config/wayvid/config.yaml

# Run wayvid-ctl
wayvid ctl status
wayvid ctl pause
```

### Arch Linux (AUR)

```bash
# Install from AUR
yay -S wayvid-git

# Or manually with makepkg
git clone https://aur.archlinux.org/wayvid-git.git
cd wayvid-git
makepkg -si
```

**Optional dependencies** for hardware acceleration:
- `mesa` - VA-API hardware video decoding
- `libva-intel-driver` - Intel GPU acceleration
- `libva-mesa-driver` - AMD GPU acceleration
- `nvidia-utils` - NVIDIA GPU acceleration

### NixOS / Nix Flakes

**Direct run** (no installation):
```bash
nix run github:YangYuS8/wayvid
```

**Install to profile**:
```bash
nix profile install github:YangYuS8/wayvid
```

**NixOS configuration**:
```nix
{
  inputs.wayvid.url = "github:YangYuS8/wayvid";
  
  outputs = { nixpkgs, wayvid, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [
        {
          environment.systemPackages = [ wayvid.packages.x86_64-linux.default ];
        }
      ];
    };
  };
}
```

See [Nix Documentation](packaging/nix/README.md) for more details.

### From Source

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
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid

# Build release binaries
cargo build --release --all-features

# Install both binaries
install -Dm755 target/release/wayvid ~/.local/bin/wayvid
install -Dm755 target/release/wayvid-ctl ~/.local/bin/wayvid-ctl

# Create config directory
mkdir -p ~/.config/wayvid

# Copy example config
cp configs/config.example.yaml ~/.config/wayvid/config.yaml

# Edit config with your video path
$EDITOR ~/.config/wayvid/config.yaml
```

### Systemd User Service

Enable automatic start on login:

```bash
# Copy systemd unit file
mkdir -p ~/.config/systemd/user
cp systemd/wayvid.service ~/.config/systemd/user/

# Enable and start
systemctl --user enable --now wayvid.service

# Check status
systemctl --user status wayvid
```

## Configuration

### Import from Wallpaper Engine

wayvid can import **Wallpaper Engine** video projects directly:

```bash
# Import a WE project directory
wayvid import ~/path/to/we-project

# Output to file
wayvid import ~/path/to/we-project --output ~/.config/wayvid/config.yaml

# Example: Workshop item
wayvid import ~/.steam/steam/steamapps/workshop/content/431960/2934567890
```

**Supported WE features**:
- ✅ Video file path detection
- ✅ Playback rate (speed)
- ✅ Volume settings
- ✅ Loop mode
- ✅ Alignment/scaling (Center/Fit/Fill/Stretch)
- ✅ Metadata (title, workshop ID, description)

The importer automatically converts WE properties to wayvid config format. See [WE Format Documentation](docs/WE_FORMAT.md) for details.

### Basic Setup

Edit `~/.config/wayvid/config.yaml`:

```yaml
# Local video file
source:
  type: File
  path: "~/Videos/wallpaper.mp4"

layout: Fill
loop: true
mute: true
volume: 0.5
playback_rate: 1.0
hwdec: true

# Power management
power:
  pause_when_hidden: true
  pause_on_battery: false
  max_fps: 30
```

### Advanced Source Types

```yaml
# HTTP/HTTPS stream
source:
  type: Url
  url: "https://example.com/video.mp4"

# RTSP stream (IP camera)
source:
  type: Rtsp
  url: "rtsp://192.168.1.100:554/stream"

# Pipe input (stdin or named pipe)
source:
  type: Pipe
  path: ""  # Empty for stdin, or "/tmp/video_pipe"

# Image sequence / slideshow
source:
  type: ImageSequence
  path: "~/Pictures/wallpapers/*.jpg"
  fps: 1.0  # One image per second
```

See [Video Sources Documentation](docs/VIDEO_SOURCES.md) for more details.

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

### Runtime Control (wayvid-ctl)

Control playback without restarting:

```bash
# Pause/Resume playback
wayvid-ctl pause                    # Pause all outputs
wayvid-ctl pause --output eDP-1     # Pause specific output
wayvid-ctl resume                   # Resume all outputs

# Seek to position
wayvid-ctl seek --output eDP-1 30.5 # Jump to 30.5 seconds

# Switch video source
wayvid-ctl switch --output HDMI-A-1 ~/Videos/new_video.mp4
wayvid-ctl switch --output eDP-1 https://example.com/stream.mp4

# Adjust playback
wayvid-ctl rate --output eDP-1 1.5  # 1.5x speed
wayvid-ctl volume --output eDP-1 0.5 # 50% volume
wayvid-ctl mute --output eDP-1       # Toggle mute

# Change layout
wayvid-ctl layout --output eDP-1 fill

# Reload configuration
wayvid-ctl reload

# Get status
wayvid-ctl status

# Quit
wayvid-ctl quit
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

### M1: Single Output MVP ✅ (Completed)
- [x] Project structure and dependencies
- [x] Layer-shell background surface
- [x] libmpv integration (simplified)
- [x] Layout calculation
- [x] Configuration system
- [x] CLI and capability check
- [x] Documentation and examples

### M2: Multi-Output & Hotplug ✅ (Completed)
- [x] Full OpenGL/EGL rendering pipeline
- [x] mpv_render_context integration
- [x] Frame callbacks and vsync
- [x] Output hotplug detection (GlobalRemove)
- [x] Per-output player management
- [x] Power saving implementation
- [x] xdg-output protocol support
- [x] Performance optimization (caching)

### M3: Runtime Control & Multi-Source ✅ (Completed)
- [x] IPC protocol design (JSON over Unix socket)
- [x] Unix socket server implementation
- [x] wayvid-ctl CLI client
- [x] Runtime control commands (pause, seek, switch, volume, layout, etc.)
- [x] Configuration hot reload (file watching)
- [x] Multi-source support (URL, RTSP, Pipe, ImageSequence)
- [x] Comprehensive documentation

### M4: WE Import & Distribution (Next)
- [ ] Wallpaper Engine project importer
- [ ] Flatpak package
- [ ] AUR package (wayvid-git)
- [ ] Nix flake
- [ ] Debian/RPM packages
- [ ] Advanced troubleshooting guide

### M5: Advanced Features (Future)
- [ ] Shared decode optimization (multi-output same video)
- [ ] Static image fallback mode
- [ ] System tray integration (optional)
- [ ] D-Bus interface
- [ ] Color management hints
- [ ] HDR planning
- [ ] Plugin system

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

## Documentation

📚 **[Complete Documentation Index](docs/README.md)**

### Quick Links
- **[Quick Start Guide](QUICKSTART.md)** - Get started in 5 minutes
- **[HDR User Guide](docs/HDR_USER_GUIDE.md)** - HDR configuration and optimization
- **[Multi-Monitor Examples](docs/MULTI_MONITOR_EXAMPLES.md)** - Advanced display setups
- **[IPC Commands](docs/IPC.md)** - Command-line control interface
- **[AI Development Prompt](AI_PROMPT.md)** - Complete project context for AI assistants

### For Developers
- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute
- **[Changelog](CHANGELOG.md)** - Version history
- **[Shared Decode Architecture](docs/SHARED_DECODE.md)** - Performance implementation details

## Contributing

Contributions are welcome! We're actively developing new features.

**Current priorities (M5):**
- Playlist support (Issue #3)
- Audio reactivity (Issue #4)
- User experience improvements (Issues #5-8)
- Platform support (Issues #9-12)

**Before contributing:**
1. Read the [Contributing Guide](CONTRIBUTING.md)
2. Check existing issues and discussions
3. For major changes, open an issue first
4. Follow Rust style guide (`cargo fmt`)
5. Add tests where applicable
6. Ensure all CI checks pass

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
A: Currently video-only. Static images may be added in future releases.

**Q: Why is CPU usage high?**  
A: Ensure hardware decode is enabled and working (`wayvid check`). Also check your video codec and resolution. For multiple outputs showing the same video, enable shared decode mode.

**Q: Can I control playback (pause/next/etc.)?**  
A: Yes! Use `wayvid-ctl` commands. See [IPC documentation](docs/IPC.md) for details.

**Q: Does this support interactive wallpapers?**  
A: No. Only video playback. No HTML/WebGL/scripts (different from Wallpaper Engine's full feature set).

**Q: How do I configure HDR content?**  
A: See the [HDR User Guide](docs/HDR_USER_GUIDE.md) for detailed configuration and troubleshooting.

**Q: Can I have different videos on different monitors?**  
A: Yes! Use per-output configuration. See [Multi-Monitor Examples](docs/MULTI_MONITOR_EXAMPLES.md).

---

**Made with ❤️ for the Wayland community**

For issues, questions, or suggestions, please [open an issue](https://github.com/yourusername/wayvid/issues).
