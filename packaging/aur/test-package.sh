#!/bin/bash
# Test script for AUR packaging

set -e

echo "🔧 Testing wayvid AUR package..."
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."
command -v makepkg >/dev/null 2>&1 || { echo "❌ makepkg not found. Install base-devel."; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ cargo not found. Install rust."; exit 1; }
echo "✅ Prerequisites OK"
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf src pkg *.pkg.tar.zst *.log
echo "✅ Cleaned"
echo ""

# Validate PKGBUILD
echo "🔍 Validating PKGBUILD..."
if command -v namcap >/dev/null 2>&1; then
    namcap PKGBUILD || echo "⚠️  namcap warnings (non-fatal)"
else
    echo "⚠️  namcap not installed, skipping validation"
fi
echo ""

# Build package
echo "📦 Building package..."
makepkg -f --noconfirm
echo "✅ Build successful"
echo ""

# Check built package
echo "🔍 Checking built package..."
PKGFILE=$(ls -t wayvid-git-*.pkg.tar.zst | head -1)
if [ -z "$PKGFILE" ]; then
    echo "❌ No package file found"
    exit 1
fi

echo "📦 Package: $PKGFILE"
echo ""

if command -v namcap >/dev/null 2>&1; then
    echo "Running namcap on package..."
    namcap "$PKGFILE" || echo "⚠️  namcap warnings (non-fatal)"
else
    echo "⚠️  namcap not installed, skipping package validation"
fi
echo ""

# List package contents
echo "📄 Package contents:"
tar -tzf "$PKGFILE" | head -20
echo ""

# Extract and check binaries
echo "🔍 Checking binaries..."
tar -xzf "$PKGFILE" -C /tmp
if [ -f /tmp/usr/bin/wayvid-gui ]; then
    echo "✅ wayvid-gui binary found"
    file /tmp/usr/bin/wayvid-gui
else
    echo "❌ wayvid-gui binary not found"
    exit 1
fi

if [ -f /tmp/usr/bin/wayvid-ctl ]; then
    echo "✅ wayvid-ctl binary found"
    file /tmp/usr/bin/wayvid-ctl
else
    echo "❌ wayvid-ctl binary not found"
    exit 1
fi
echo ""

# Check dependencies
echo "🔗 Checking binary dependencies..."
ldd /tmp/usr/bin/wayvid-gui | grep -E "(libmpv|libwayland|libc)" || true
echo ""

# Cleanup
rm -rf /tmp/usr

echo "✅ All tests passed!"
echo ""
echo "To install the package:"
echo "  sudo pacman -U $PKGFILE"
echo ""
echo "Or to install with makepkg:"
echo "  makepkg -si"
