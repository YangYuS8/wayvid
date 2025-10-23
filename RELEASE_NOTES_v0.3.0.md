# wayvid v0.3.0 - M4 Complete: WE Import & Distribution

**Release Date**: 2025-10-23

Milestone 4 brings **Wallpaper Engine project import** and **comprehensive Linux distribution support**. wayvid is now ready for wide distribution across all major Linux platforms!

---

## 🎯 Major Features

### 1️⃣ Wallpaper Engine Import 🎨

Import your Wallpaper Engine video projects directly:

```bash
# Import WE project
wayvid import ~/.steam/steam/steamapps/workshop/content/431960/YOUR_WORKSHOP_ID \
  --output ~/.config/wayvid/config.yaml

# Or from local project
wayvid import ~/path/to/we-project --output ~/.config/wayvid/config.yaml
```

**Supported WE Properties**:
- ✅ Video file path detection
- ✅ Playback rate (speed)
- ✅ Volume settings
- ✅ Loop mode
- ✅ Alignment/scaling (Center/Fit/Fill/Stretch → Centre/Contain/Cover/Fill)
- ✅ Metadata (title, workshop ID, description)

Automatic conversion means you can use your existing WE wallpapers on Wayland without manual configuration!

### 2️⃣ Universal Linux Distribution 📦

**AppImage** (Recommended):
```bash
# Download and run - works on ANY Linux distribution
wget https://github.com/YangYuS8/wayvid/releases/download/v0.3.0/wayvid-0.3.0-x86_64.AppImage
chmod +x wayvid-0.3.0-x86_64.AppImage
./wayvid-0.3.0-x86_64.AppImage
```

**Arch Linux (AUR)**:
```bash
yay -S wayvid-git
```

**NixOS / Nix Flakes**:
```bash
# Direct run
nix run github:YangYuS8/wayvid

# Install to profile
nix profile install github:YangYuS8/wayvid
```

---

## 📋 What's New

### Added

#### WE Project Parser & Importer
- Complete Wallpaper Engine `project.json` parser
- CLI `import` command for WE → wayvid conversion
- Property mapping with automatic conversion
- Metadata preservation
- Full documentation in `docs/WE_FORMAT.md` (409 lines)

#### Multi-Platform Packaging

**AppImage** (Universal Linux):
- Self-contained binary with all dependencies
- Works on any distribution (Ubuntu, Fedora, Arch, Debian, etc.)
- No root required, no installation needed
- Includes both `wayvid` and `wayvid-ctl`
- Automated CI/CD build pipeline

**AUR** (Arch Linux):
- `wayvid-git` - Development version
- `wayvid` - Stable releases
- Complete dependency management
- Optional hardware acceleration packages
- Automated testing and validation

**Nix Flakes**:
- Modern Nix package with rust-overlay
- Multiple installation methods (run/profile/NixOS/Home Manager)
- Complete development shell
- Integration examples for NixOS and Home Manager
- 351-line comprehensive documentation

#### Documentation Overhaul
- **README.md**: Complete rewrite of installation section
  - AppImage as primary installation method
  - WE import instructions
  - Multi-platform installation guides
  - Updated status to M4 Complete
- **QUICKSTART.md**: Streamlined quick start with WE import
- **CHANGELOG.md**: Detailed version history
- **Packaging guides**: Complete docs for AUR, Nix, AppImage

### Fixed

- **serde enum deserialization**: Fixed Combo/Slider variant ordering bug
  - Issue: `alignment` property was incorrectly parsed (e.g., `2` → Contain instead of Cover)
  - Solution: Reordered enum variants (Combo before Slider)
  - Impact: All WE property conversions now work correctly

### Changed

- **Version**: 0.1.0 → 0.3.0
- **Repository**: Updated to `https://github.com/YangYuS8/wayvid`
- **Status**: M3 → M4 Complete

---

## 📦 Downloads

### AppImage (Universal Linux)

| Architecture | Download | Size | SHA256 |
|--------------|----------|------|--------|
| x86_64 | [wayvid-0.3.0-x86_64.AppImage](https://github.com/YangYuS8/wayvid/releases/download/v0.3.0/wayvid-0.3.0-x86_64.AppImage) | ~15-20 MB | *See SHA256SUMS* |

**Note**: AppImage will be built automatically by GitHub Actions. If not available immediately, wait a few minutes for CI to complete.

### Source Code

- **Source tarball**: [wayvid-0.3.0.tar.gz](https://github.com/YangYuS8/wayvid/archive/refs/tags/v0.3.0.tar.gz)
- **Source zip**: [wayvid-0.3.0.zip](https://github.com/YangYuS8/wayvid/archive/refs/tags/v0.3.0.zip)

---

## 🚀 Quick Start

### 1. Install

**AppImage** (Any Linux):
```bash
wget https://github.com/YangYuS8/wayvid/releases/download/v0.3.0/wayvid-0.3.0-x86_64.AppImage
chmod +x wayvid-0.3.0-x86_64.AppImage
sudo mv wayvid-0.3.0-x86_64.AppImage /usr/local/bin/wayvid
```

**Arch Linux**:
```bash
yay -S wayvid-git
```

**Nix**:
```bash
nix profile install github:YangYuS8/wayvid
```

### 2. Configure

**From Wallpaper Engine**:
```bash
wayvid import ~/path/to/we-project --output ~/.config/wayvid/config.yaml
```

**Or manually**:
```bash
mkdir -p ~/.config/wayvid
cat > ~/.config/wayvid/config.yaml << 'EOF'
source:
  type: File
  path: "~/Videos/wallpaper.mp4"
layout: Fill
loop: true
mute: true
hwdec: true
EOF
```

### 3. Run

```bash
wayvid run
```

### 4. Control

```bash
# Pause/Resume
wayvid-ctl pause
wayvid-ctl resume

# Switch video
wayvid-ctl switch ~/Videos/another.mp4

# Adjust playback
wayvid-ctl rate 1.5
wayvid-ctl volume 0.5
```

---

## 📚 Documentation

- **README**: [Main documentation](https://github.com/YangYuS8/wayvid/blob/main/README.md)
- **Quick Start**: [QUICKSTART.md](https://github.com/YangYuS8/wayvid/blob/main/docs/QUICKSTART.md)
- **WE Import**: [WE_FORMAT.md](https://github.com/YangYuS8/wayvid/blob/main/docs/WE_FORMAT.md)
- **Video Sources**: [VIDEO_SOURCES.md](https://github.com/YangYuS8/wayvid/blob/main/docs/VIDEO_SOURCES.md)
- **IPC API**: [IPC.md](https://github.com/YangYuS8/wayvid/blob/main/docs/IPC.md)

**Packaging Guides**:
- [AUR Packaging](https://github.com/YangYuS8/wayvid/blob/main/packaging/aur/README.md)
- [Nix Flakes](https://github.com/YangYuS8/wayvid/blob/main/packaging/nix/README.md)
- [AppImage](https://github.com/YangYuS8/wayvid/blob/main/packaging/appimage/README.md)

---

## ✅ Supported Compositors

| Compositor | Status | Tested Version |
|------------|--------|----------------|
| **Hyprland** | ✅ Primary | v0.35+ |
| **niri** | ✅ Primary | Latest git |
| Sway | 🟡 Should work | Uses wlr-layer-shell |
| River | 🟡 Should work | Uses wlr-layer-shell |
| KDE/GNOME | ❌ Not supported | No wlr-layer-shell |

---

## 🔧 System Requirements

**Minimum**:
- Wayland compositor with `wlr-layer-shell` support
- libmpv (video playback)
- OpenGL/EGL support
- Linux kernel 5.10+

**Recommended**:
- Hardware video decode (VA-API/NVDEC)
- 4+ GB RAM
- Multi-core CPU

**Optional** (Hardware Acceleration):
- Intel: `intel-media-driver`, `libva-intel-driver`
- AMD: `mesa-va-drivers`
- NVIDIA: `nvidia-utils`, `nvidia-vaapi-driver`

---

## 🐛 Known Issues

- KDE/GNOME not supported (no `wlr-layer-shell` protocol)
- Some NVIDIA drivers may require `hwdec: false` in config
- AppImage requires FUSE (or use `--appimage-extract-and-run`)

---

## 🙏 Credits

Thanks to all contributors and testers who made this release possible!

**Technologies**:
- [mpv](https://mpv.io/) - Video playback
- [Smithay](https://github.com/Smithay/smithay) - Wayland client toolkit
- [Rust](https://www.rust-lang.org/) - Programming language

---

## 📝 Full Changelog

See [CHANGELOG.md](https://github.com/YangYuS8/wayvid/blob/main/CHANGELOG.md) for complete version history.

---

## 🔮 What's Next (M5)

Future milestones will focus on:
- Performance optimizations (shared decode, memory efficiency)
- Advanced features (HDR support, color management)
- Extended compositor support
- Community features

---

**Enjoy your dynamic video wallpapers on Wayland!** 🎉

If you encounter issues, please [open an issue](https://github.com/YangYuS8/wayvid/issues/new) on GitHub.
