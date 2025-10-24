#!/bin/bash
# Test script for frame skip intelligence feature

set -e

echo "========================================="
echo "Frame Skip Intelligence Test"
echo "========================================="
echo ""

# Build the project
echo "🔨 Building project..."
cargo build --release

echo ""
echo "📊 Testing frame skip behavior..."
echo ""
echo "This test will simulate high CPU load to trigger frame skipping."
echo "Expected behavior:"
echo "  1. Normal load: No frame skipping"
echo "  2. High load: Enter skip mode after 3 consecutive overload frames"
echo "  3. Recovery: Exit skip mode after load decreases"
echo ""

# Create a CPU stress command that runs in background
stress_cpu() {
    local duration=$1
    echo "⚠️  Simulating CPU stress for ${duration}s..."
    # Run CPU-intensive task in background
    timeout ${duration}s sh -c 'while true; do :; done' &
    local pid=$!
    echo "   Stress PID: $pid"
    echo $pid
}

# Check if we have a test video
TEST_VIDEO="${HOME}/Videos/test.mp4"
if [ ! -f "$TEST_VIDEO" ]; then
    echo "⚠️  Test video not found at: $TEST_VIDEO"
    echo "   Using default video (if available)"
    TEST_VIDEO=""
fi

# Create test config with FPS limit
TEST_CONFIG=$(mktemp /tmp/wayvid-test-XXXXXX.yaml)
cat > "$TEST_CONFIG" << EOF
# Test configuration for frame skip testing
power:
  max_fps: 60  # Set target to 60 FPS
  pause_when_hidden: false
  pause_on_battery: false

# Default video source (will be overridden if TEST_VIDEO exists)
source:
  File: "${TEST_VIDEO:-/tmp/test.mp4}"

layout: Fill
loop: true
mute: true
hwdec: true
EOF

echo "📄 Test config created at: $TEST_CONFIG"
echo ""

# Test 1: Normal operation (10 seconds)
echo "Test 1: Normal operation (no stress)"
echo "Expected: No frame skipping, load < 80%"
echo ""
timeout 10s ./target/release/wayvid run --config "$TEST_CONFIG" 2>&1 | \
    grep -E "(Frame stats|Frame skip|SKIP MODE)" || true
echo ""

# Test 2: With CPU stress (simulated overload)
echo "Test 2: With simulated CPU stress"
echo "Expected: Enter skip mode when overloaded"
echo ""

# Start CPU stress
STRESS_PID=$(stress_cpu 15)

# Run wayvid for 20 seconds
timeout 20s ./target/release/wayvid run --config "$TEST_CONFIG" 2>&1 | \
    grep -E "(Frame stats|Frame skip|SKIP MODE|Entering skip mode|Exiting skip mode)" || true

# Kill stress if still running
if ps -p $STRESS_PID > /dev/null 2>&1; then
    kill $STRESS_PID 2>/dev/null || true
fi

echo ""
echo "Test 3: Recovery after stress"
echo "Expected: Exit skip mode after load decreases"
echo ""

# Run for another 10 seconds after stress ends
timeout 10s ./target/release/wayvid run --config "$TEST_CONFIG" 2>&1 | \
    grep -E "(Frame stats|Frame skip|SKIP MODE|Exiting skip mode)" || true

echo ""
echo "========================================="
echo "✅ Frame skip intelligence test complete"
echo "========================================="
echo ""
echo "Check the log output above for:"
echo "  - 📊 Frame stats: Shows frame counts and skip rate"
echo "  - 🔴 Entering skip mode: Triggered when load > 80%"
echo "  - 🟢 Exiting skip mode: Triggered when load < 60%"
echo "  - [SKIP MODE]: Indicates currently in skip mode"
echo ""

# Cleanup
rm -f "$TEST_CONFIG"
