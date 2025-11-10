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
cargo build --release
sudo cp target/release/wayvid /usr/local/bin/
sudo cp target/release/wayvid-ctl /usr/local/bin/
```

### Optional: GUI
```bash
cargo build --release --features gui
sudo cp target/release/wayvid-gui /usr/local/bin/
```

## Verify Installation
```bash
wayvid --version
wayvid-ctl --version
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
