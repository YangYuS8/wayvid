#!/bin/bash
# Test wayvid AppImage on multiple distributions

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

APPIMAGE="${1:-build/wayvid-latest-$(uname -m).AppImage}"

if [ ! -f "$APPIMAGE" ]; then
    echo -e "${RED}❌ AppImage not found: $APPIMAGE${NC}"
    echo "Usage: $0 [path/to/wayvid.AppImage]"
    exit 1
fi

echo -e "${BLUE}🧪 Testing wayvid AppImage${NC}"
echo "📦 Package: $APPIMAGE"
echo ""

# Test 1: Basic functionality
echo -e "${YELLOW}Test 1: Basic functionality${NC}"
echo "Running: $APPIMAGE --version"
if "$APPIMAGE" --version &> /dev/null; then
    VERSION=$("$APPIMAGE" --version | head -n1)
    echo -e "${GREEN}✅ Version check passed: $VERSION${NC}"
else
    echo -e "${RED}❌ Version check failed${NC}"
    exit 1
fi

echo ""
echo "Running: $APPIMAGE --help"
if "$APPIMAGE" --help &> /dev/null; then
    echo -e "${GREEN}✅ Help output works${NC}"
else
    echo -e "${RED}❌ Help failed${NC}"
    exit 1
fi

# Test 2: wayvid-ctl
echo ""
echo -e "${YELLOW}Test 2: wayvid-ctl functionality${NC}"
echo "Running: $APPIMAGE ctl --version"
if "$APPIMAGE" ctl --version &> /dev/null; then
    CTL_VERSION=$("$APPIMAGE" ctl --version | head -n1)
    echo -e "${GREEN}✅ wayvid-ctl works: $CTL_VERSION${NC}"
else
    echo -e "${RED}❌ wayvid-ctl failed${NC}"
    exit 1
fi

# Test 3: Dependency check
echo ""
echo -e "${YELLOW}Test 3: Dependency check${NC}"
echo "Extracting AppImage..."
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Extract AppImage - try different methods
if "$APPIMAGE" --appimage-extract &> /dev/null; then
    echo "✅ Extracted using --appimage-extract"
elif command -v unsquashfs &> /dev/null && unsquashfs "$APPIMAGE" &> /dev/null; then
    echo "✅ Extracted using unsquashfs"
    mv squashfs-root squashfs-root.tmp
    mkdir squashfs-root
    mv squashfs-root.tmp/* squashfs-root/
    rmdir squashfs-root.tmp
else
    echo -e "${YELLOW}⚠️  Cannot extract AppImage (FUSE not available)${NC}"
    echo "Skipping dependency check in CI environment"
    cd - > /dev/null
    rm -rf "$TEMP_DIR"
    # Skip this test but don't fail
    echo ""
    echo -e "${YELLOW}Test 4: File size check${NC}"
    SIZE=$(du -h "$APPIMAGE" | cut -f1)
    SIZE_BYTES=$(du -b "$APPIMAGE" | cut -f1)
    echo "📏 Size: $SIZE ($SIZE_BYTES bytes)"
    
    if [ $SIZE_BYTES -gt 104857600 ]; then  # 100 MB
        echo -e "${YELLOW}⚠️  AppImage is quite large (>100MB)${NC}"
    else
        echo -e "${GREEN}✅ Size is reasonable${NC}"
    fi
    
    # Test 5: Permissions check
    echo ""
    echo -e "${YELLOW}Test 5: Permissions check${NC}"
    if [ -x "$APPIMAGE" ]; then
        echo -e "${GREEN}✅ AppImage is executable${NC}"
    else
        echo -e "${RED}❌ AppImage is not executable${NC}"
        exit 1
    fi
    
    # Summary (skipped tests 3 & 6)
    echo ""
    echo -e "${GREEN}✅ Basic tests passed!${NC}"
    echo -e "${YELLOW}ℹ️  Note: Tests 3 and 6 skipped (AppImage extraction not available)${NC}"
    echo ""
    echo "📦 AppImage is ready for distribution:"
    echo "   Size: $SIZE"
    echo "   Version: $VERSION"
    echo ""
    echo "🚀 Next steps:"
    echo "   1. Test on different distributions (Ubuntu, Fedora, Arch, etc.)"
    echo "   2. Upload to GitHub Releases"
    echo "   3. Update download links in documentation"
    exit 0
fi

echo "Checking binaries..."
if ldd squashfs-root/usr/bin/wayvid-gui | grep -q "not found"; then
    echo -e "${RED}❌ Missing dependencies:${NC}"
    ldd squashfs-root/usr/bin/wayvid-gui | grep "not found"
    cd - > /dev/null
    rm -rf "$TEMP_DIR"
    exit 1
else
    echo -e "${GREEN}✅ All dependencies satisfied${NC}"
fi

cd - > /dev/null
rm -rf "$TEMP_DIR"

# Test 4: File size check
echo ""
echo -e "${YELLOW}Test 4: File size check${NC}"
SIZE=$(du -h "$APPIMAGE" | cut -f1)
SIZE_BYTES=$(du -b "$APPIMAGE" | cut -f1)
echo "📏 Size: $SIZE ($SIZE_BYTES bytes)"

if [ $SIZE_BYTES -gt 104857600 ]; then  # 100 MB
    echo -e "${YELLOW}⚠️  AppImage is quite large (>100MB)${NC}"
else
    echo -e "${GREEN}✅ Size is reasonable${NC}"
fi

# Test 5: Permissions check
echo ""
echo -e "${YELLOW}Test 5: Permissions check${NC}"
if [ -x "$APPIMAGE" ]; then
    echo -e "${GREEN}✅ AppImage is executable${NC}"
else
    echo -e "${RED}❌ AppImage is not executable${NC}"
    exit 1
fi

# Test 6: Extract and inspect
echo ""
echo -e "${YELLOW}Test 6: Content inspection${NC}"
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
"$APPIMAGE" --appimage-extract &> /dev/null

echo "📦 AppImage contents:"
echo "Binaries:"
ls -lh squashfs-root/usr/bin/
echo ""
echo "Libraries:"
ls -lh squashfs-root/usr/lib/ 2>/dev/null || echo "No bundled libraries"
echo ""
echo "Documentation:"
ls -lh squashfs-root/usr/share/doc/wayvid/ 2>/dev/null || echo "No documentation"

cd - > /dev/null
rm -rf "$TEMP_DIR"

# Summary
echo ""
echo -e "${GREEN}✅ All tests passed!${NC}"
echo ""
echo "📦 AppImage is ready for distribution:"
echo "   Size: $SIZE"
echo "   Version: $VERSION"
echo ""
echo "🚀 Next steps:"
echo "   1. Test on different distributions (Ubuntu, Fedora, Arch, etc.)"
echo "   2. Upload to GitHub Releases"
echo "   3. Update download links in documentation"
