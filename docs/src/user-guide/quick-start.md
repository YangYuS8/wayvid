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
```bash
wayvid
```

## Auto-Start

### Hyprland
Add to `~/.config/hypr/hyprland.conf`:
```
exec-once = wayvid
```

### Niri
Add to `~/.config/niri/config.kdl`:
```kdl
spawn-at-startup "wayvid"
```

### Sway
Add to `~/.config/sway/config`:
```
exec wayvid
```

## Next Steps

- [Configuration Guide](./configuration.md) - Customize settings
- [Video Sources](./video-sources.md) - Learn about source types
- [Multi-Monitor](./multi-monitor.md) - Set up multiple displays
