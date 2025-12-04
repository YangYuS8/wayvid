# wayvid

<div align="center">
<img src="logo.svg" alt="wayvid logo" width="100" height="100">
</div>

Animated wallpaper manager for Wayland compositors.

**v0.5** introduces a GUI-first design — just open the app, pick a wallpaper, and apply.

## Features

- 🖼️ **GUI wallpaper browser** - Thumbnails, search, and one-click apply
- 🖥️ **Multi-monitor support** - Independent wallpapers per display
- ⚡ **Hardware accelerated** - VA-API/NVDEC via mpv
- 🎮 **Steam Workshop** - Import video wallpapers from Wallpaper Engine
- 🌈 **HDR support** - 10-bit HDR with tone-mapping
- 🔧 **CLI tools** - `wayvid-ctl` for scripting and automation

## Tested Compositors

- Hyprland ✓
- Niri ✓
- Sway (should work)
- River (should work)

## Quick Start

```bash
# Install (Arch)
yay -S wayvid

# Run
wayvid-gui
```

## Quick Links

- [Installation](./user-guide/installation.md)
- [GUI Guide](./user-guide/gui.md)
- [CLI Reference](./reference/cli.md)
- [GitHub](https://github.com/YangYuS8/wayvid)

## Requirements

- Linux with Wayland
- Compositor with wlr-layer-shell support
- libmpv, libEGL, libwayland-client

## License

MIT OR Apache-2.0
