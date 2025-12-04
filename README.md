<div align="center">

<img src="logo.svg" alt="wayvid logo" width="100" height="100">

# wayvid

Animated wallpaper manager for Wayland

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.5.0--alpha.1-green.svg)](https://github.com/YangYuS8/wayvid/releases)

[Documentation](https://yangyus8.top/wayvid/) • [Releases](https://github.com/YangYuS8/wayvid/releases)

</div>

## What it does

wayvid plays video files as animated wallpapers on Wayland compositors. **v0.5** introduces a GUI-first design — just open the app, pick a wallpaper, and apply.

**Features:**

- 🖼️ **GUI wallpaper browser** with thumbnails and search
- 🖥️ **Multi-monitor support** with independent wallpapers per display
- ⚡ **Hardware accelerated** decoding (VA-API/NVDEC via mpv)
- 🎮 **Steam Workshop** import (video wallpapers)
- 🌈 **HDR support** with tone-mapping
- 🔧 **CLI tools** for scripting and automation

**Tested on:** Hyprland, Niri

**Should work on:** Sway, River, and other wlr-layer-shell compositors

## Demo

<!-- TODO: Add demo video/GIF here -->
*Demo coming soon*

## Install

### Arch Linux (AUR)

```bash
yay -S wayvid
```

### Nix

```bash
nix profile install github:YangYuS8/wayvid
```

### From source

```bash
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid
cargo build --release
sudo install -Dm755 target/release/wayvid-gui /usr/local/bin/
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/
```

**Dependencies:** libmpv, libEGL, libwayland-client

## Usage

### GUI (Recommended)

```bash
wayvid-gui
```

The GUI provides:
- Wallpaper library browser with thumbnails
- Monitor selection and preview
- Settings configuration (autostart, power management)
- Minimizes to system tray

### CLI Control

```bash
# Apply wallpaper
wayvid-ctl apply ~/Videos/wallpaper.mp4
wayvid-ctl apply ~/Videos/wallpaper.mp4 --output DP-1

# Control playback
wayvid-ctl pause
wayvid-ctl resume
wayvid-ctl stop

# Check status
wayvid-ctl status
wayvid-ctl status --json

# List monitors
wayvid-ctl outputs
```

### Autostart

The GUI includes an autostart option in Settings. Alternatively:

**Niri/Hyprland config:**
```kdl
# niri: ~/.config/niri/config.kdl
spawn-at-startup "wayvid-gui" "--minimized"
```

```conf
# hyprland: ~/.config/hypr/hyprland.conf
exec-once = wayvid-gui --minimized
```

**systemd (optional):**
```bash
systemctl --user enable --now wayvid
```

## Configuration

Settings are managed through the GUI and saved automatically to:
```
~/.config/wayvid/settings.yaml
```

For advanced users, legacy config.yaml is still supported:
```yaml
# ~/.config/wayvid/config.yaml
source:
  type: file
  path: ~/Videos/wallpaper.mp4
layout: fill
volume: 0
```

## Multi-monitor

Use the GUI's Monitor tab, or configure per-output:

```yaml
# ~/.config/wayvid/config.yaml
source:
  type: file
  path: ~/Videos/default.mp4

per_output:
  DP-1:
    source:
      type: file
      path: ~/Videos/left.mp4
  HDMI-A-1:
    source:
      type: file
      path: ~/Videos/right.mp4
```

## Steam Workshop

Import video wallpapers from Wallpaper Engine through the GUI, or:

```bash
wayvid-ctl apply ~/.steam/steam/steamapps/workshop/content/431960/<id>/video.mp4
```

**Note:** Only video wallpapers are supported. Web/scene types require Wallpaper Engine.

## Troubleshooting

**Black screen:**
```bash
mpv ~/Videos/wallpaper.mp4  # Test if video plays
```

**High CPU:**
```yaml
# Enable hardware decode
hwdec: true
```

**View logs:**
```bash
journalctl --user -u wayvid -f
```

## Project Structure

```
crates/
├── wayvid-core     # Core types and configuration
├── wayvid-engine   # Video rendering engine (Wayland + MPV)
├── wayvid-library  # Wallpaper library (SQLite + thumbnails)
├── wayvid-gui      # GUI application (iced framework)
└── wayvid-ctl      # CLI control tool
```

## Contributing

```bash
cargo build --release
cargo test --workspace
cargo clippy --workspace
```

## License

MIT OR Apache-2.0

## Acknowledgments

Built with [mpv](https://mpv.io/), [wayland-rs](https://github.com/Smithay/wayland-rs), and [iced](https://github.com/iced-rs/iced).
