# wayvid - Wayland Video Wallpaper Engine

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Documentation](https://img.shields.io/badge/docs-mdbook-blue)](https://www.yangyus8.top/wayvid/)

A dynamic video wallpaper engine for Wayland compositors, with priority support for **Hyprland** and **niri**. Compatible with Wallpaper Engine's video wallpaper parameters.

> 📖 **[Read Full Documentation](https://www.yangyus8.top/wayvid/)** | **[中文文档](https://www.yangyus8.top/wayvid/zh-cn/)**

## Features

- ✅ **GUI Control Panel** - Full-featured graphical interface with Workshop integration
- ✅ **Native Wayland** - Uses `wlr-layer-shell` for background layer placement
- ✅ **Full Input Passthrough** - Wallpaper doesn't interfere with desktop interaction
- ✅ **Multi-Monitor** - Per-output configuration with hotplug support
- ✅ **Hardware Decode** - VA-API/NVDEC support with software fallback
- ✅ **Flexible Layouts** - Fill, Contain, Stretch, Cover, Centre modes
- ✅ **Multi-Source Support** - Local files, HTTP/RTSP streams, pipes, image sequences
- ✅ **Runtime Control** - CLI via `wayvid-ctl` + GUI control panel
- ✅ **Hot Reload** - Config changes applied instantly without restart
- ✅ **Power Management** - Battery detection, FPS limiting, auto-pause
- ✅ **OpenGL Rendering** - Full EGL/OpenGL integration with mpv render API
- ✅ **Low Resource** - Efficient playback with intelligent caching
- ✅ **HDR Support** - Automatic HDR10/HLG detection with tone mapping



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
- ✅ **Steam Workshop Integration** - One-click import from WE Workshop ([#23](https://github.com/YangYuS8/wayvid/issues/23))
- 🔜 **Niri Optimizations** - Workspace-aware, scroll-optimized ([#24](https://github.com/YangYuS8/wayvid/issues/24))
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

### Quick Install

```bash
# Arch Linux
yay -S wayvid-git

# NixOS
nix profile install github:YangYuS8/wayvid

# From source
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid && cargo build --release --all-features
sudo install -Dm755 target/release/{wayvid,wayvid-ctl,wayvid-gui} /usr/local/bin/
```

📖 **Detailed instructions**: [Installation Guide](https://www.yangyus8.top/wayvid/user-guide/installation.html)

## Configuration

Basic config (`~/.config/wayvid/config.yaml`):

```yaml
outputs:
  default:
    source:
      type: file
      path: ~/Videos/wallpaper.mp4
    layout: fill
    volume: 0
```

📖 **Full guide**: [Configuration Reference](https://www.yangyus8.top/wayvid/user-guide/configuration.html) • [Multi-Monitor](https://www.yangyus8.top/wayvid/user-guide/multi-monitor.html)

## Usage

```bash
# Run
wayvid

# GUI Control Panel (NEW!)
wayvid-gui

# CLI Control
wayvid-ctl play
wayvid-ctl pause
wayvid-ctl set-volume 50

# Workshop (one-command install)
wayvid workshop install <id> -o ~/.config/wayvid/config.yaml

# Autostart (Hyprland)
echo "exec-once = wayvid" >> ~/.config/hypr/hyprland.conf
```

📖 **Full commands**: [GUI Guide](https://www.yangyus8.top/wayvid/user-guide/gui.html) • [CLI Reference](https://www.yangyus8.top/wayvid/reference/cli.html) • [IPC Control](https://www.yangyus8.top/wayvid/features/ipc.html) • [Workshop](https://www.yangyus8.top/wayvid/features/workshop.html)

## Troubleshooting

```bash
# Check system
wayvid check

# Debug
wayvid --log-level debug
```

Common fixes:
- Black screen → Check video path, run `mpv <file>` to verify
- High CPU → Enable `hwdec: true`, check `vainfo`
- No hwdec → Install `libva-intel-driver` (Intel) or `nvidia-vaapi-driver` (NVIDIA)

📖 See full [troubleshooting guide](https://www.yangyus8.top/wayvid/user-guide/installation.html#troubleshooting)



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



## Development

```bash
cargo build --release
cargo test
cargo clippy
```

📖 See [Developer Guide](https://www.yangyus8.top/wayvid/dev/building.html) • [Architecture](https://www.yangyus8.top/wayvid/dev/architecture.html) • [Contributing](https://www.yangyus8.top/wayvid/dev/contributing.html)

## Documentation

📚 **[Full Documentation](https://www.yangyus8.top/wayvid/)** - Built with [mdBook](https://rust-lang.github.io/mdBook/)

### Quick Start
```bash
# Install (Arch Linux)
yay -S wayvid

# Configure
mkdir -p ~/.config/wayvid
nano ~/.config/wayvid/config.yaml

# Run
wayvid &
wayvid-ctl play
```

See [Quick Start Guide](https://www.yangyus8.top/wayvid/user-guide/quick-start.html) or [快速开始 (中文)](https://www.yangyus8.top/wayvid/zh-cn/user-guide/quick-start.html).

### Documentation Structure
- **User Guide**: [Installation](https://www.yangyus8.top/wayvid/user-guide/installation.html) • [Configuration](https://www.yangyus8.top/wayvid/user-guide/configuration.html) • [Multi-Monitor](https://www.yangyus8.top/wayvid/user-guide/multi-monitor.html)
- **Features**: [HDR](https://www.yangyus8.top/wayvid/features/hdr.html) • [Workshop](https://www.yangyus8.top/wayvid/features/workshop.html) • [IPC](https://www.yangyus8.top/wayvid/features/ipc.html) • [Niri](https://www.yangyus8.top/wayvid/features/niri.html)
- **Developer**: [Building](https://www.yangyus8.top/wayvid/dev/building.html) • [Workflow](https://www.yangyus8.top/wayvid/dev/workflow.html) • [Architecture](https://www.yangyus8.top/wayvid/dev/architecture.html) • [Contributing](https://www.yangyus8.top/wayvid/dev/contributing.html)
- **Reference**: [Config](https://www.yangyus8.top/wayvid/reference/config.html) • [CLI](https://www.yangyus8.top/wayvid/reference/cli.html) • [IPC Protocol](https://www.yangyus8.top/wayvid/reference/ipc-protocol.html) • [WE Format](https://www.yangyus8.top/wayvid/reference/we-format.html)

Build docs locally:
```bash
cd docs && mdbook serve --open
```

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

**Q: Why is CPU usage high?**  
A: Enable hardware decode: `hwdec: true`. Check: `wayvid check`

**Q: Can I control playback?**  
A: Yes: `wayvid-ctl pause/resume/seek/switch`

**Q: Per-monitor videos?**  
A: Yes, via `per_output` config. See [docs](https://www.yangyus8.top/wayvid/user-guide/multi-monitor.html).

**Q: Wallpaper Engine support?**  
A: Video wallpapers only (no HTML/WebGL). Use `wayvid workshop import <id>`.

---

**Made with ❤️ for the Wayland community**

🐛 [Issues](https://github.com/YangYuS8/wayvid/issues) • 💬 [Discussions](https://github.com/YangYuS8/wayvid/discussions) • 📖 [Docs](https://www.yangyus8.top/wayvid/)
