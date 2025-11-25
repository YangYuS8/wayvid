<div align="center">

# wayvid

**Dynamic Video Wallpaper Engine for Wayland**

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.4.2-green.svg)](https://github.com/YangYuS8/wayvid/releases)
[![Documentation](https://img.shields.io/badge/docs-mdbook-blue)](https://www.yangyus8.top/wayvid/)

*Bring your desktop to life with smooth video wallpapers on Wayland*

[📖 Documentation](https://www.yangyus8.top/wayvid/) • [🚀 Quick Start](#-quick-start) • [💬 Discussions](https://github.com/YangYuS8/wayvid/discussions) • [🐛 Issues](https://github.com/YangYuS8/wayvid/issues)

---

</div>

## ✨ Why wayvid?

wayvid is a **production-ready** video wallpaper solution designed specifically for modern Wayland compositors. Unlike traditional wallpaper tools, wayvid offers:

- 🎬 **Native Wayland Support** - Built on `wlr-layer-shell`, works seamlessly with Hyprland, Niri, Sway, and more
- 🖥️ **True Multi-Monitor** - Independent videos per display with hotplug support and intelligent scaling
- ⚡ **Hardware Accelerated** - VA-API/NVDEC support with 60% less CPU usage on multi-display setups
- 🎨 **HDR Ready** - Full 10-bit HDR pipeline with automatic tone-mapping for SDR displays
- 🎮 **Steam Workshop** - One-click import of Wallpaper Engine wallpapers (video only)
- 🔧 **Easy to Manage** - GUI control panel, CLI tools, or systemd service - your choice
- 🔋 **Power Efficient** - Intelligent FPS throttling, battery detection, and workspace-aware rendering

**Supported Compositors:** Hyprland ✅ | Niri ✅ | Sway 🟡 | River 🟡 | KDE/GNOME ❌

---

## 🚀 Quick Start

Get wayvid running in under 5 minutes:

### 1. Install

<details>
<summary><b>Arch Linux (AUR)</b></summary>

```bash
yay -S wayvid
```

</details>

<details>
<summary><b>NixOS / Nix</b></summary>

```bash
nix profile install github:YangYuS8/wayvid
```

</details>

<details>
<summary><b>From Source</b></summary>

```bash
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid
cargo build --release --all-features
sudo install -Dm755 target/release/{wayvid,wayvid-ctl,wayvid-gui} /usr/local/bin/
```

</details>

### 2. Configure

Create `~/.config/wayvid/config.yaml`:

```yaml
outputs:
  default:
    source:
      type: file
      path: ~/Videos/your-wallpaper.mp4
    layout: fill
    volume: 0
```

### 3. Run

**Option A: systemd (recommended)**

```bash
systemctl --user enable --now wayvid.service
```

**Option B: GUI control panel**

```bash
wayvid-gui  # Click "Start Daemon"
```

**Option C: Compositor autostart**

```bash
# Hyprland: ~/.config/hypr/hyprland.conf
exec-once = systemctl --user start wayvid.service

# Niri: ~/.config/niri/config.kdl
spawn-at-startup "systemctl" "--user" "start" "wayvid.service"
```

That's it! Your video wallpaper should now be running. 🎉

➡️ **Next steps:** [Configuration Guide](https://www.yangyus8.top/wayvid/user-guide/configuration.html) • [Multi-Monitor Setup](https://www.yangyus8.top/wayvid/user-guide/multi-monitor.html) • [Steam Workshop](https://www.yangyus8.top/wayvid/features/workshop.html)

---

## 📦 Usage & Management

### Daemon Control

```bash
# Start/stop/restart daemon
wayvid daemon start
wayvid daemon stop
wayvid daemon restart

# Check status and view logs
wayvid daemon status
wayvid daemon logs --follow
```

### GUI Control Panel

```bash
wayvid-gui
```

The GUI provides:

- 🚀 One-click daemon start/stop
- 📝 Visual config editor with YAML validation
- 🎮 Steam Workshop browser
- 🖥️ Per-monitor configuration
- 📊 Real-time status monitoring

### Runtime Control (CLI)

```bash
# Playback control
wayvid-ctl play
wayvid-ctl pause
wayvid-ctl status

# Change settings on-the-fly
wayvid-ctl set-volume 50
wayvid-ctl set-source ~/Videos/new-wallpaper.mp4

# Reload configuration
wayvid-ctl reload-config
```

### Steam Workshop

Import Wallpaper Engine wallpapers with one command:

```bash
# List local Workshop items
wayvid workshop list

# Import a specific wallpaper
wayvid workshop import <item-id> -o ~/.config/wayvid/config.yaml
```

---

## 🎯 Key Features

<table>
<tr>
<td width="50%">

---

## 📚 Documentation

| Topic                          | Link                                                                              |
| ------------------------------ | --------------------------------------------------------------------------------- |
| 📖**Full Documentation** | [yangyus8.top/wayvid](https://www.yangyus8.top/wayvid/)                              |
| 🚀 Quick Start                 | [Installation Guide](https://www.yangyus8.top/wayvid/user-guide/installation.html)   |
| ⚙️ Configuration             | [Config Reference](https://www.yangyus8.top/wayvid/user-guide/configuration.html)    |
| 🖥️ Multi-Monitor             | [Multi-Monitor Guide](https://www.yangyus8.top/wayvid/user-guide/multi-monitor.html) |
| 🎮 Workshop                    | [Workshop Integration](https://www.yangyus8.top/wayvid/features/workshop.html)       |
| 🔧 CLI Reference               | [Command Reference](https://www.yangyus8.top/wayvid/reference/cli.html)              |
| 🏗️ Development               | [Developer Guide](https://www.yangyus8.top/wayvid/dev/building.html)                 |

---

## 🛠️ Troubleshooting

**Problem: Black screen**

```bash
# Verify video file works
mpv ~/Videos/wallpaper.mp4

# Check system capabilities
wayvid check
```

**Problem: High CPU usage**

```yaml
# Enable hardware decode in config.yaml
outputs:
  default:
    hwdec: true
```

**Problem: Daemon not starting**

```bash
# Check daemon status
wayvid daemon status

# View logs
wayvid daemon logs --follow
```

More help: [Troubleshooting Guide](https://www.yangyus8.top/wayvid/user-guide/installation.html#troubleshooting)

---

## 🤝 Contributing

Contributions are welcome! Whether it's bug reports, feature requests, or code contributions.

**Getting Started:**

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and test thoroughly
4. Submit a pull request

**Development:**

```bash
# Build and run
cargo build --release --all-features
cargo test
cargo clippy

# Build documentation
cd docs && mdbook serve --open
```

See [Contributing Guide](https://www.yangyus8.top/wayvid/dev/contributing.html) and [Developer Guide](https://www.yangyus8.top/wayvid/dev/building.html) for details.

---

## 📄 License

wayvid is dual-licensed under your choice of:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

---

## ☕ Support the Project

If you find wayvid useful, consider supporting its development:

[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support%20Me-FF5E5B?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/yangyus8)

Your support helps maintain and improve wayvid! Every contribution is appreciated. 💙

---

## 🙏 Acknowledgments

wayvid is built on top of excellent open-source projects:

- [**mpv**](https://mpv.io/) - Powerful media player library
- [**Hyprland**](https://hyprland.org/) & [**niri**](https://github.com/YaLTeR/niri) - Modern Wayland compositors
- [**wlr-layer-shell**](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) - Wayland layer shell protocol
- [**Wallpaper Engine**](https://www.wallpaperengine.io/) - Inspiration and format compatibility

---

<div align="center">

**Made with ❤️ for the Wayland community**

[⭐ Star on GitHub](https://github.com/YangYuS8/wayvid) • [📖 Documentation](https://www.yangyus8.top/wayvid/) • [💬 Discussions](https://github.com/YangYuS8/wayvid/discussions)

</div>
