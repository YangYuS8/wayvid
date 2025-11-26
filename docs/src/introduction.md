# wayvid

**A high-performance video wallpaper engine for Wayland compositors**

wayvid brings video wallpapers to Wayland, supporting HDR, multi-monitor setups, and Steam Workshop integration.

## Key Features

- 🎬 **Multiple Video Sources**: Local files, directories, Steam Workshop
- 🖥️ **Multi-Monitor**: Independent or synchronized playback per display
- 🌈 **HDR Support**: Full 10-bit HDR pipeline with automatic tone-mapping
- 🎮 **Steam Workshop**: Direct import of Wallpaper Engine video projects
- 🪟 **Niri Optimized**: First-class support for Niri compositor with workspace awareness
- 🔌 **IPC Control**: Real-time control via CLI/GUI
- ⚡ **Performance**: Hardware-accelerated (VA-API/NVDEC), low CPU/memory usage
- 🖼️ **Wallpaper Engine Style GUI**: Intuitive control panel with bottom monitor bar

## Quick Links

- [Quick Start Guide](./user-guide/quick-start.md) - Get started in 5 minutes
- [Installation](./user-guide/installation.md) - Install on your system
- [Configuration](./user-guide/configuration.md) - Customize settings
- [GUI Guide](./user-guide/gui.md) - Control panel usage
- [GitHub Repository](https://github.com/YangYuS8/wayvid)

## System Requirements

- **OS**: Linux with Wayland compositor
- **Compositor**: Hyprland, Niri (full support), Sway, River (partial support)
- **GPU**: Any with EGL and OpenGL ES 3.0 support
- **Libraries**: libmpv, libEGL, libwayland-client

## Recent Improvements

### v0.4.4+
- **Optimized Frame Rendering**: Proper Wayland frame callback handling, reduced CPU usage by ~40%
- **Wallpaper Engine Style GUI**: Redesigned interface with bottom monitor selector and unified wallpaper library
- **Click-to-Apply**: Single click to select, double-click to apply wallpapers
- **Shared Decoder**: Memory-efficient multi-monitor support with frame sharing

## License

wayvid is dual-licensed under MIT OR Apache-2.0.
