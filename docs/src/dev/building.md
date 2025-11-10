# Building from Source

## Prerequisites

### System Dependencies

**Arch Linux:**
```bash
sudo pacman -S rust mpv wayland wayland-protocols libxkbcommon fontconfig
```

**Ubuntu/Debian:**
```bash
sudo apt install cargo libmpv-dev libwayland-dev wayland-protocols \
                 libxkbcommon-dev libfontconfig1-dev libegl1-mesa-dev
```

**Fedora:**
```bash
sudo dnf install cargo mpv-libs-devel wayland-devel wayland-protocols-devel \
                 libxkbcommon-devel fontconfig-devel mesa-libEGL-devel
```

## Clone Repository

```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid
```

## Build

### Release Build
```bash
cargo build --release
```

Binaries in `target/release/`:
- `wayvid` - Main daemon
- `wayvid-ctl` - CLI control tool

### With GUI
```bash
cargo build --release --all-features
```

Additional binary:
- `wayvid-gui` - Desktop GUI

### Development Build
```bash
cargo build
```

Faster compilation, debug symbols, in `target/debug/`.

## Feature Flags

```bash
# Workshop support
cargo build --features workshop

# Niri integration
cargo build --features niri

# GUI
cargo build --features gui

# All features
cargo build --all-features
```

## Install

```bash
sudo cp target/release/wayvid{,-ctl,-gui} /usr/local/bin/
```

Or use cargo:
```bash
cargo install --path . --all-features
```

## Run Tests

```bash
cargo test --all-features
```

## Verify

```bash
wayvid --version
wayvid-ctl --version
wayvid-gui --version  # if built with --features gui
```

## Clean Build

```bash
cargo clean
```

## Optimization

For maximum performance:
```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Troubleshooting

**Missing dependencies:**
- Check error message for missing library
- Install corresponding `-dev` or `-devel` package

**Linker errors:**
- Install `clang` or `gcc`
- Ensure all system libraries present

**Build takes too long:**
- Use `cargo build` (debug) for development
- Enable `mold` linker (see [Development Workflow](./workflow.md))
