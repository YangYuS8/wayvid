# Installation

## Arch Linux (AUR)

```bash
yay -S wayvid-git
```

## Nix / NixOS

### Direct run (no installation)

```bash
# Run wayvid-gui directly from GitHub
nix run github:YangYuS8/wayvid

# Run wayvid-ctl
nix run github:YangYuS8/wayvid#wayvid-ctl -- status
```

### Install to profile

```bash
nix profile install github:YangYuS8/wayvid
```

### NixOS configuration

Add to your `flake.nix`:

```nix
{
  inputs.wayvid.url = "github:YangYuS8/wayvid";
  
  # In your system configuration:
  environment.systemPackages = [
    inputs.wayvid.packages.${system}.default
  ];
}
```

### Home Manager

```nix
home.packages = [
  inputs.wayvid.packages.${system}.default
];
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
sudo pacman -S rust mpv wayland wayland-protocols libxkbcommon fontconfig mesa

# Ubuntu/Debian
sudo apt install cargo libmpv-dev libwayland-dev libxkbcommon-dev libfontconfig-dev libegl-dev

# Fedora
sudo dnf install cargo mpv-libs-devel wayland-devel libxkbcommon-devel fontconfig-devel mesa-libEGL-devel
```

### Build

```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid
cargo build --release
```

### Install (User)

Using the installation script (recommended):

```bash
./scripts/install.sh --user
```

This installs to `~/.local/bin/` and adds a desktop entry.

### Install (System)

```bash
sudo ./scripts/install.sh --system
```

### Manual Install

```bash
# User install
install -Dm755 target/release/wayvid-gui ~/.local/bin/wayvid-gui
install -Dm755 target/release/wayvid-ctl ~/.local/bin/wayvid-ctl
install -Dm644 packaging/wayvid-gui.desktop ~/.local/share/applications/wayvid.desktop
install -Dm644 logo.svg ~/.local/share/icons/hicolor/scalable/apps/wayvid.svg

# System install
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
# Using uninstall script
./scripts/uninstall.sh --user

# AUR
yay -R wayvid-git

# Nix
nix profile remove wayvid

# Manual cleanup
rm -rf ~/.config/wayvid ~/.cache/wayvid
```
