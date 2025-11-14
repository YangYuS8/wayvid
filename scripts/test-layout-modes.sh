#!/usr/bin/env bash
# Test different layout modes to verify video scaling behavior
# Usage: ./scripts/test-layout-modes.sh <video-path>

set -e

VIDEO_PATH="${1:-$HOME/.steam/steam/steamapps/workshop/content/431960/*/video.mp4}"
VIDEO_PATH=$(echo $VIDEO_PATH | head -n1)

if [ ! -f "$VIDEO_PATH" ]; then
    echo "❌ Video file not found: $VIDEO_PATH"
    echo ""
    echo "Usage: $0 <video-path>"
    echo "Example: $0 ~/Videos/wallpaper.mp4"
    exit 1
fi

echo "🎬 Testing Layout Modes with: $VIDEO_PATH"
echo ""

TEST_DIR="/tmp/wayvid-layout-test"
mkdir -p "$TEST_DIR"

# Test each layout mode
MODES=("Fill" "Contain" "Stretch" "Centre")

for mode in "${MODES[@]}"; do
    echo "================================"
    echo "Testing mode: $mode"
    echo "================================"
    
    # Create config
    cat > "$TEST_DIR/test-$mode.yaml" << EOF
source:
  type: File
  path: $VIDEO_PATH
layout: $mode
loop: true
mute: true
hwdec: true
EOF
    
    echo "Config: $TEST_DIR/test-$mode.yaml"
    echo ""
    echo "Expected behavior:"
    case "$mode" in
        Fill)
            echo "  - Video should fill entire screen"
            echo "  - No black bars visible"
            echo "  - May crop edges if aspect ratio differs"
            echo "  - Uses MPV panscan=1.0"
            ;;
        Contain)
            echo "  - Video fits inside screen"
            echo "  - Full video content visible"
            echo "  - May show black bars (letterbox/pillarbox)"
            echo "  - Uses MPV panscan=0.0"
            ;;
        Stretch)
            echo "  - Video fills entire screen"
            echo "  - No black bars, no cropping"
            echo "  - May appear distorted"
            echo "  - Uses MPV keepaspect=no"
            ;;
        Centre)
            echo "  - Video at original size"
            echo "  - Centered on screen"
            echo "  - Black bars if video smaller than screen"
            echo "  - Uses MPV video-unscaled=yes"
            ;;
    esac
    echo ""
    
    echo "Press Enter to start test (Ctrl+C to skip)..."
    read -r
    
    # Run for 5 seconds
    echo "Running wayvid for 5 seconds..."
    timeout 5 wayvid run --config "$TEST_DIR/test-$mode.yaml" || true
    
    echo ""
    echo "✓ Test complete for $mode"
    echo ""
    sleep 1
done

echo "================================"
echo "All tests complete!"
echo "================================"
echo ""
echo "Config files saved to: $TEST_DIR"
echo "You can re-run specific modes with:"
echo "  wayvid run --config $TEST_DIR/test-Fill.yaml"
echo ""
echo "Recommended mode for wallpapers: Fill"
echo "  - No black bars"
echo "  - Maintains aspect ratio"
echo "  - Crops edges to fill screen (like Wallpaper Engine)"
