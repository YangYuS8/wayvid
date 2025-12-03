# wayvid

<div align="center">
<img src="logo.svg" alt="wayvid logo" width="100" height="100">
</div>

Video wallpaper daemon for Wayland compositors.

## Features

- **Multi-monitor** - Independent video per display
- **Hardware decode** - VA-API/NVDEC via mpv
- **Steam Workshop** - Import video wallpapers from Wallpaper Engine
- **HDR** - 10-bit HDR with tone-mapping
- **GUI + CLI** - Control panel and command-line tools

## Tested Compositors

- Hyprland ✓
- Niri ✓
- Sway (should work)
- River (should work)

## Quick Links

- [Installation](./user-guide/installation.md)
- [Configuration](./user-guide/configuration.md)
- [GitHub](https://github.com/YangYuS8/wayvid)

## Requirements

- Linux with Wayland
- Compositor with wlr-layer-shell support
- libmpv, libEGL, libwayland-client

## License

MIT OR Apache-2.0
