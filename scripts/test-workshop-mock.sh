#!/bin/bash
# Test Workshop functionality with mock data

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "======================================"
echo "Workshop Integration Mock Test"
echo "======================================"
echo

# Build if needed
if [ ! -f "target/release/wayvid" ]; then
    echo "Building wayvid..."
    cargo build --release
fi

WAYVID="$PROJECT_ROOT/target/release/wayvid"
TEST_DIR="/tmp/wayvid-workshop-test"
CACHE_DIR="$HOME/.cache/wayvid/workshop"

# Cleanup
cleanup() {
    echo "Cleaning up test files..."
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

# Create test directory
mkdir -p "$TEST_DIR"

echo "✓ Built wayvid"
echo "✓ Created test directory: $TEST_DIR"
echo

# Test 1: Steam detection
echo "Test 1: Steam Installation Detection"
echo "-------------------------------------"
if $WAYVID workshop list --log-level warn 2>&1 | grep -q "Found"; then
    echo "✓ Steam detection works"
else
    echo "✗ Steam detection failed"
    exit 1
fi
echo

# Test 2: Cache directory creation
echo "Test 2: Cache Directory Management"
echo "-------------------------------------"
$WAYVID workshop cache --log-level warn > /dev/null 2>&1
if [ -d "$CACHE_DIR" ]; then
    echo "✓ Cache directory created: $CACHE_DIR"
else
    echo "✗ Cache directory creation failed"
    exit 1
fi
echo

# Test 3: Help messages
echo "Test 3: CLI Help Messages"
echo "-------------------------------------"
for cmd in list info search download install import cache; do
    if $WAYVID workshop $cmd --help > /dev/null 2>&1; then
        echo "✓ 'workshop $cmd' help works"
    else
        echo "✗ 'workshop $cmd' help failed"
        exit 1
    fi
done
echo

# Test 4: Create mock Workshop structure (for testing parser)
echo "Test 4: Workshop Project Parser"
echo "-------------------------------------"
MOCK_WORKSHOP="$TEST_DIR/mock_workshop/123456789"
mkdir -p "$MOCK_WORKSHOP"

# Create mock project.json (minimal valid format)
cat > "$MOCK_WORKSHOP/project.json" <<'EOF'
{
  "title": "Test Video Wallpaper",
  "description": "A test wallpaper for wayvid",
  "file": "scene.mp4",
  "type": "video"
}
EOF

# Create a dummy video file
# Just use a placeholder file for testing
touch "$MOCK_WORKSHOP/scene.mp4"
echo "✓ Created mock Workshop item structure"

# Test importing the mock project
if $WAYVID import "$MOCK_WORKSHOP" --log-level error 2>&1 | grep -q "source:"; then
    echo "✓ Workshop project parsing works"
else
    echo "✗ Workshop project parsing failed"
    echo "Debug output:"
    $WAYVID import "$MOCK_WORKSHOP" --log-level warn 2>&1 | head -20
    exit 1
fi
echo

# Test 5: Configuration generation
echo "Test 5: Configuration Generation"
echo "-------------------------------------"
CONFIG_OUTPUT="$TEST_DIR/test-config.yaml"
$WAYVID import "$MOCK_WORKSHOP" -o "$CONFIG_OUTPUT" --log-level warn > /dev/null 2>&1

if [ -f "$CONFIG_OUTPUT" ]; then
    echo "✓ Configuration file generated"
    echo "  Output: $CONFIG_OUTPUT"
    
    # Check for File (capital F) and the video path
    if grep -q "type: File" "$CONFIG_OUTPUT" && grep -q "scene.mp4" "$CONFIG_OUTPUT"; then
        echo "✓ Configuration contains correct video path"
    else
        echo "✗ Configuration validation failed"
        echo "Expected 'type: File' and 'scene.mp4' in config"
        exit 1
    fi
else
    echo "✗ Configuration generation failed"
    exit 1
fi
echo

# Summary
echo "======================================"
echo "✅ All tests passed!"
echo "======================================"
echo
echo "Summary:"
echo "  • Steam detection: OK"
echo "  • Cache management: OK"
echo "  • CLI commands: OK"
echo "  • Project parsing: OK"
echo "  • Config generation: OK"
echo
echo "Next steps to test with real Workshop items:"
echo "  1. Open Steam and subscribe to Wallpaper Engine wallpapers"
echo "  2. Run: $WAYVID workshop list"
echo "  3. Import: $WAYVID workshop import <id> -o config.yaml"
echo
