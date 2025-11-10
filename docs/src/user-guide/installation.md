# Installation

## Arch Linux

### Stable (AUR)
```bash
yay -S wayvid
```

### Development (AUR)
```bash
yay -S wayvid-git
```

**Note:** Both packages include all three binaries:
- `wayvid` - Main daemon
- `wayvid-ctl` - Command-line control tool
- `wayvid-gui` - Desktop GUI control panel (requires X11/Wayland session)

## From Source

### Prerequisites
```bash
# Arch/Manjaro
sudo pacman -S rust mpv wayland wayland-protocols

# Ubuntu/Debian  
sudo apt install cargo libmpv-dev libwayland-dev libxkbcommon-dev

# Fedora
sudo dnf install cargo mpv-libs-devel wayland-devel libxkbcommon-devel
```

### Build & Install
```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid

# Build with all features (including GUI)
cargo build --release --all-features

# Install all binaries
sudo install -Dm755 target/release/wayvid /usr/local/bin/wayvid
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/wayvid-ctl
sudo install -Dm755 target/release/wayvid-gui /usr/local/bin/wayvid-gui
```

## Verify Installation
```bash
wayvid --version
wayvid-ctl --version
wayvid-gui --version  # GUI control panel
```

## Uninstall

### AUR
```bash
yay -R wayvid
```

### Manual
```bash
sudo rm /usr/local/bin/wayvid*
rm -rf ~/.config/wayvid
```
