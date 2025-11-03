Milestone 4 brings **Wallpaper Engine import** and **comprehensive Linux distribution support**.

## 🎯 Highlights

### Wallpaper Engine Import 🎨
Import your WE video projects directly:
```bash
wayvid import ~/.steam/steamapps/workshop/content/431960/YOUR_WORKSHOP_ID \
  --output ~/.config/wayvid/config.yaml
```

### Multi-Platform Distribution 📦
- **AppImage**: Universal Linux binary (any distro) - *Building via CI*
- **AUR**: `yay -S wayvid-git` (Arch Linux)
- **Nix**: `nix run github:YangYuS8/wayvid` (NixOS)

## ✅ What's New

**WE Import**:
- Complete Wallpaper Engine `project.json` parser
- Automatic property conversion (alignment, rate, volume, loop)
- Metadata preservation (title, workshop ID, description)
- Full documentation in `docs/WE_FORMAT.md`

**Packaging**:
- AppImage with automated CI/CD builds
- AUR packages (wayvid-git + wayvid stable)
- Nix Flakes with rust-overlay
- Complete packaging documentation

**Documentation**:
- README.md: Complete rewrite of installation section
- QUICKSTART.md: Streamlined quick start
- CHANGELOG.md: Detailed version history

**Bug Fixes**:
- Fixed WE property enum deserialization (Combo/Slider ordering)

## 📦 Installation

**AppImage** (Any Linux - Recommended):
```bash
wget https://github.com/YangYuS8/wayvid/releases/download/v0.3.0/wayvid-0.3.0-x86_64.AppImage
chmod +x wayvid-0.3.0-x86_64.AppImage
sudo mv wayvid-0.3.0-x86_64.AppImage /usr/local/bin/wayvid
```

**Arch Linux**:
```bash
yay -S wayvid-git
```

**NixOS**:
```bash
nix profile install github:YangYuS8/wayvid
```

## 📚 Documentation

- [README](https://github.com/YangYuS8/wayvid#readme)
- [Quick Start](https://github.com/YangYuS8/wayvid/blob/main/docs/QUICKSTART.md)
- [WE Import Guide](https://github.com/YangYuS8/wayvid/blob/main/docs/WE_FORMAT.md)
- [Full Changelog](https://github.com/YangYuS8/wayvid/blob/main/CHANGELOG.md)

**Packaging Guides**:
- [AUR](https://github.com/YangYuS8/wayvid/blob/main/packaging/aur/README.md)
- [Nix](https://github.com/YangYuS8/wayvid/blob/main/packaging/nix/README.md)
- [AppImage](https://github.com/YangYuS8/wayvid/blob/main/packaging/appimage/README.md)

---

**Note**: AppImage will be automatically uploaded when CI build completes.
