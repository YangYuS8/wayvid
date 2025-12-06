# Building

## Dependencies

**Arch:**
```bash
sudo pacman -S rust mpv wayland wayland-protocols libxkbcommon fontconfig mesa
```

**Ubuntu/Debian:**
```bash
sudo apt install cargo libmpv-dev libwayland-dev libxkbcommon-dev libfontconfig-dev libegl-dev
```

**Fedora:**
```bash
sudo dnf install cargo mpv-libs-devel wayland-devel libxkbcommon-devel fontconfig-devel mesa-libEGL-devel
```

## Build

```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid

# Release build
cargo build --release

# Install using script (recommended)
./scripts/install.sh --user

# Or manual install
sudo install -Dm755 target/release/wayvid-gui /usr/local/bin/
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/
```

## Binaries

v0.5 produces two binaries:
- `wayvid-gui` - Main GUI application with embedded playback engine
- `wayvid-ctl` - CLI control tool for scripting

## Test

```bash
cargo test --workspace
cargo clippy --workspace
```

## Verify

```bash
wayvid-gui --version
wayvid-ctl --version
```
