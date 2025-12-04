# Installation

## Arch Linux (AUR)

```bash
yay -S wayvid
```

## Nix

```bash
nix profile install github:YangYuS8/wayvid
```

## AppImage

```bash
# Download
wget https://github.com/YangYuS8/wayvid/releases/latest/download/wayvid-x86_64.AppImage
chmod +x wayvid-x86_64.AppImage

# Run
./wayvid-x86_64.AppImage
```

## From Source

### Dependencies

```bash
# Arch
sudo pacman -S rust mpv wayland wayland-protocols libxkbcommon fontconfig

# Ubuntu/Debian
sudo apt install cargo libmpv-dev libwayland-dev libxkbcommon-dev libfontconfig-dev

# Fedora
sudo dnf install cargo mpv-libs-devel wayland-devel libxkbcommon-devel fontconfig-devel
```

### Build

```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid
cargo build --release
```

### Install

```bash
sudo install -Dm755 target/release/wayvid-gui /usr/local/bin/wayvid-gui
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/wayvid-ctl
```

## Verify

```bash
wayvid-gui --version
wayvid-ctl --version
```

## Uninstall

```bash
# AUR
yay -R wayvid

# Manual
sudo rm /usr/local/bin/wayvid-gui /usr/local/bin/wayvid-ctl
rm -rf ~/.config/wayvid ~/.cache/wayvid
```
