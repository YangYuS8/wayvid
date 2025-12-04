<div align="center">

<img src="logo.svg" alt="wayvid logo" width="100" height="100">

# wayvid

Video wallpaper daemon for Wayland

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.4.5--alpha.2-green.svg)](https://github.com/YangYuS8/wayvid/releases)

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
# Option 1: Direct (simplest)
wayvid run

# Option 2: GUI
wayvid-gui

# Option 3: systemd (for auto-start, see below)
systemctl --user start wayvid
```

### Niri Autostart (Recommended for Niri users)

Following [niri's systemd setup guide](https://yalter.github.io/niri/Example-systemd-Setup.html):

```bash
# Install service file (skip if installed via package manager)
mkdir -p ~/.config/systemd/user
cp systemd/wayvid.service ~/.config/systemd/user/
systemctl --user daemon-reload

# Add wayvid to niri startup
systemctl --user add-wants niri.service wayvid.service
```

This creates a link in `~/.config/systemd/user/niri.service.wants/`. wayvid will automatically start with niri and stop when niri exits.

**Alternative:** If you prefer not using systemd, add to your `~/.config/niri/config.kdl`:

```kdl
spawn-at-startup "wayvid" "run"
```

### Control

```bash
# Check daemon status and playback info
wayvid-ctl status

# Pause/resume playback
wayvid-ctl pause                     # Pause all outputs
wayvid-ctl resume                    # Resume all outputs
wayvid-ctl pause -o DP-1             # Pause specific output

# Switch video source
wayvid-ctl switch ~/Videos/new.mp4   # Switch on first output (single monitor)
wayvid-ctl switch -o DP-1 ~/Videos/new.mp4  # Switch on specific output

# Adjust volume and playback
wayvid-ctl volume -o DP-1 0.5        # Set volume (0.0-1.0)
wayvid-ctl rate -o DP-1 1.5          # Set playback speed
wayvid-ctl seek -o DP-1 30.0         # Seek to 30 seconds

# Reload config without restart
wayvid-ctl reload
```

**Tip:** Use `wayvid-ctl status` to see available output names (e.g., DP-1, HDMI-A-1, eDP-1).

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

**Daemon not responding / "Failed to connect to daemon":**

```bash
# Check if daemon is running
wayvid daemon status

# If using systemd
systemctl --user status wayvid

# If not running, start it
systemctl --user start wayvid
# Or run directly:
wayvid run
```

**Niri: wayvid not starting automatically:**

```bash
# Check if add-wants is set up correctly
ls ~/.config/systemd/user/niri.service.wants/wayvid.service

# If missing, run:
systemctl --user add-wants niri.service wayvid.service

# Check logs for errors
journalctl --user -u wayvid -f
```

**Config file permission issues:**

```bash
# Ensure config file is readable
chmod 644 ~/.config/wayvid/config.yaml
ls -la ~/.config/wayvid/
```

**Black screen:**

```bash
mpv ~/Videos/wallpaper.mp4  # Test if video plays
wayvid check                # Check system capabilities
```

**High CPU:**

```yaml
# Enable hardware decode in config
hwdec: true
```

**View daemon logs:**

```bash
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
