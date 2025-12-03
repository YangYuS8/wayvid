# Installation

## Arch Linux (AUR)

```bash
yay -S wayvid
```

## Nix

```bash
nix profile install github:YangYuS8/wayvid
```

## From Source

### Dependencies

```bash
# Arch
sudo pacman -S rust mpv wayland wayland-protocols

# Ubuntu/Debian
sudo apt install cargo libmpv-dev libwayland-dev libxkbcommon-dev

# Fedora
sudo dnf install cargo mpv-libs-devel wayland-devel libxkbcommon-devel
```

### Build

```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid
cargo build --release --all-features
```

### Install

```bash
sudo install -Dm755 target/release/wayvid /usr/local/bin/wayvid
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/wayvid-ctl
sudo install -Dm755 target/release/wayvid-gui /usr/local/bin/wayvid-gui
```

## Verify

```bash
wayvid --version
wayvid-ctl --version
wayvid-gui --version
```

## Uninstall

```bash
# AUR
yay -R wayvid

# Manual
sudo rm /usr/local/bin/wayvid*
rm -rf ~/.config/wayvid
```
