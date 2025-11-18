# Quick Start

Get wayvid running in 5 minutes.

## Install

### Arch Linux (AUR)
```bash
yay -S wayvid-git
```

### From Source
```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid
cargo install --path .
```

## Basic Usage

### 1. Create Config
```bash
wayvid --generate-config
```

This creates `~/.config/wayvid/config.yaml`.

### 2. Set Video Path
Edit config:
```yaml
source:
  type: file
  path: ~/Videos/wallpaper.mp4
```

### 3. Run

**Choose one of the following methods:**

#### Method 1: Direct Run (Simple)
```bash
wayvid run
```

#### Method 2: systemd Service (Recommended)
```bash
# Enable and start service
systemctl --user enable wayvid.service
systemctl --user start wayvid.service

# Check status
systemctl --user status wayvid.service

# View logs
journalctl --user -u wayvid -f
```

#### Method 3: GUI Control Panel
```bash
# Launch GUI (will prompt to start daemon if needed)
wayvid-gui
```

## Auto-Start

### Method 1: systemd (Recommended)
**Best for**: All compositors, easy management, auto-restart on failure

```bash
# One-time setup
systemctl --user enable wayvid.service

# Starts automatically on login
```

**Advantages:**
- ✅ Automatic restart on crash
- ✅ Easy management (`systemctl start/stop/restart`)
- ✅ Centralized logging (`journalctl`)
- ✅ Resource limits and security

### Method 2: Compositor Config (Alternative)

#### Hyprland
Add to `~/.config/hypr/hyprland.conf`:
```
# Option A: Direct spawn
exec-once = wayvid run

# Option B: Via systemd (recommended)
exec-once = systemctl --user start wayvid.service
```

#### Niri
Add to `~/.config/niri/config.kdl`:
```kdl
// Option A: Direct spawn
spawn-at-startup "wayvid" "run"

// Option B: Via systemd (recommended)
spawn-at-startup "systemctl" "--user" "start" "wayvid.service"
```

#### Sway
Add to `~/.config/sway/config`:
```
# Option A: Direct spawn
exec wayvid run

# Option B: Via systemd (recommended)
exec systemctl --user start wayvid.service
```

### Method 3: GUI Auto-Start
Launch `wayvid-gui` and use the **"Start Daemon"** button. The GUI can manage the daemon without terminal commands.

## Management Commands

```bash
# Start/stop/restart daemon
systemctl --user start wayvid.service
systemctl --user stop wayvid.service
systemctl --user restart wayvid.service

# Check daemon status
systemctl --user status wayvid.service

# View real-time logs
journalctl --user -u wayvid -f

# Control playback (requires daemon running)
wayvid-ctl play
wayvid-ctl pause
wayvid-ctl status
```

## Next Steps

- [Configuration Guide](./configuration.md) - Customize settings
- [Video Sources](./video-sources.md) - Learn about source types
- [Multi-Monitor](./multi-monitor.md) - Set up multiple displays
