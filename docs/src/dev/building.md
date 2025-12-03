# Building

## Dependencies

**Arch:**
```bash
sudo pacman -S rust mpv wayland wayland-protocols libxkbcommon
```

**Ubuntu/Debian:**
```bash
sudo apt install cargo libmpv-dev libwayland-dev libxkbcommon-dev
```

**Fedora:**
```bash
sudo dnf install cargo mpv-libs-devel wayland-devel libxkbcommon-devel
```

## Build

```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid

# Release build with all features
cargo build --release --all-features

# Install
sudo install -Dm755 target/release/{wayvid,wayvid-ctl,wayvid-gui} /usr/local/bin/
```

## Features

```bash
cargo build --features gui       # GUI only
cargo build --features workshop  # Workshop only
cargo build --all-features       # Everything
```

## Test

```bash
cargo test
cargo clippy
```

## Verify

```bash
wayvid --version
```
