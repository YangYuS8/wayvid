#!/usr/bin/env bash
# Development capability check script

set -e

echo "=== wayvid Development Environment Check (v0.5) ==="
echo

# Check Rust
echo "[Rust Toolchain]"
if command -v rustc &> /dev/null; then
    echo "  ✓ rustc: $(rustc --version)"
    echo "  ✓ cargo: $(cargo --version)"
else
    echo "  ✗ Rust not installed"
    echo "    Install from: https://rustup.rs"
    exit 1
fi

# Check Wayland
echo
echo "[Wayland Environment]"
if [ -n "$WAYLAND_DISPLAY" ]; then
    echo "  ✓ WAYLAND_DISPLAY: $WAYLAND_DISPLAY"
else
    echo "  ⚠ WAYLAND_DISPLAY not set (not running under Wayland)"
fi

if [ -n "$XDG_CURRENT_DESKTOP" ]; then
    echo "  ℹ Desktop: $XDG_CURRENT_DESKTOP"
fi

# Check system libraries
echo
echo "[System Libraries]"

check_lib() {
    if ldconfig -p | grep -q "$1"; then
        echo "  ✓ $1"
        return 0
    else
        echo "  ✗ $1 not found"
        return 1
    fi
}

check_lib "libwayland-client"
check_lib "libmpv"
check_lib "libEGL"
check_lib "libGL"

# Check tools
echo
echo "[Tools]"

check_cmd() {
    if command -v "$1" &> /dev/null; then
        echo "  ✓ $1"
        return 0
    else
        echo "  ℹ $1 not found (optional)"
        return 1
    fi
}

check_cmd "mpv"
check_cmd "vainfo"
check_cmd "vdpauinfo"
check_cmd "wayland-info" || check_cmd "weston-info"

# Build check
echo
echo "[Build Check]"
if cargo check --workspace --quiet 2>/dev/null; then
    echo "  ✓ Workspace compiles successfully"
else
    echo "  ✗ Workspace has compilation errors"
    echo "    Run 'cargo check --workspace' for details"
fi

# Workspace info
echo
echo "[Workspace Crates]"
echo "  - wayvid-core     (core types and config)"
echo "  - wayvid-engine   (Wayland + MPV rendering)"
echo "  - wayvid-library  (SQLite wallpaper library)"
echo "  - wayvid-gui      (iced GUI - main entry)"
echo "  - wayvid-ctl      (CLI control tool)"

echo
echo "=== Check Complete ==="
echo
echo "To build and run:"
echo "  cargo build --release"
echo "  ./target/release/wayvid-gui"
echo "  ./target/release/wayvid-ctl status"
