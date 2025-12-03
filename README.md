<div align="center">

<img src="logo.svg" alt="wayvid logo" width="100" height="100">

# wayvid

Video wallpaper daemon for Wayland

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.4.4-green.svg)](https://github.com/YangYuS8/wayvid/releases)

[Documentation](https://yangyus8.top/wayvid/) • [Releases](https://github.com/YangYuS8/wayvid/releases)

</div>

## What it does

wayvid plays video files as wallpapers on Wayland compositors using wlr-layer-shell protocol.

**Features:**

- Multi-monitor support with independent video per display
- Hardware accelerated decoding (VA-API/NVDEC via mpv)
- Steam Workshop import (video wallpapers only)
- HDR support with tone-mapping
- GUI control panel and CLI tools

**Tested on:** Hyprland, Niri

**Should work on:** Sway, River, and other wlr-layer-shell compositors

## Demo

<!-- TODO: Add demo video/GIF here
<p align="center">
  <img src="assets/demo.gif" alt="wayvid demo" width="800">
</p>
-->

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
cargo build --release --all-features
sudo install -Dm755 target/release/{wayvid,wayvid-ctl,wayvid-gui} /usr/local/bin/
```

**Dependencies:** libmpv, libEGL, libwayland-client

## Usage

### 1. Create config

```bash
mkdir -p ~/.config/wayvid
cat > ~/.config/wayvid/config.yaml << 'EOF'
source:
  type: file
  path: ~/Videos/wallpaper.mp4
layout: fill
volume: 0
EOF
```

### 2. Run

```bash
# Option 1: systemd (recommended)
systemctl --user enable --now wayvid.service

# Option 2: Direct
wayvid run

# Option 3: GUI
wayvid-gui
```

### Control

```bash
wayvid-ctl status
wayvid-ctl pause
wayvid-ctl play
wayvid-ctl set-source ~/Videos/new.mp4
```

## Multi-monitor

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

Import video wallpapers from Wallpaper Engine:

```bash
wayvid workshop list              # List subscribed items
wayvid workshop import <id>       # Generate config
```

**Note:** Only video wallpapers are supported. Web/scene types don't work.

## Troubleshooting

**Black screen:**

```bash
mpv ~/Videos/wallpaper.mp4  # Test if video plays
wayvid check                # Check system capabilities
```

**High CPU:**

```yaml
# Enable hardware decode
hwdec: true
```

**Daemon issues:**

```bash
wayvid daemon status
wayvid daemon logs --follow
```

## Project structure

```
wayvid      - Main daemon
wayvid-ctl  - CLI control tool
wayvid-gui  - Desktop GUI
```

## Contributing

```bash
cargo build --release --all-features
cargo test
cargo clippy
```

## License

MIT OR Apache-2.0

## Acknowledgments

Built with [mpv](https://mpv.io/), [wayland-rs](https://github.com/Smithay/wayland-rs), and [egui](https://github.com/emilk/egui).
