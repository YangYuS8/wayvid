#!/bin/bash
# HDR Tone Mapping Test Script for wayvid
# ========================================
#
# This script helps test different tone mapping algorithms and settings

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
VIDEO_PATH="${1:-}"
DURATION="${2:-5}"
LOG_LEVEL="${3:-info}"

# Function to print colored output
print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_info() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if video path provided
if [ -z "$VIDEO_PATH" ]; then
    print_error "Usage: $0 <video-path> [duration=5] [log-level=info]"
    echo ""
    echo "Example:"
    echo "  $0 /path/to/hdr-video.mp4 10 debug"
    echo ""
    echo "To download HDR test videos:"
    echo "  # HDR10 test pattern (small)"
    echo "  wget https://4kmedia.org/lg-hdr-demo-new-york-hdr-uhd-4k/"
    echo ""
    echo "  # HLG BBC test"
    echo "  wget https://download.opencontent.netflix.com/TechblogAssets/Chimera/Mobile_Chimera-AVC_12500_23976fps_10bit_HDR_HLG_EBU3689-1Kpbs.mp4"
    exit 1
fi

# Check if video exists
if [ ! -f "$VIDEO_PATH" ]; then
    print_error "Video file not found: $VIDEO_PATH"
    exit 1
fi

print_header "HDR Tone Mapping Test"
print_info "Video: $VIDEO_PATH"
print_info "Test duration: ${DURATION}s per algorithm"
print_info "Log level: $LOG_LEVEL"
echo ""

# Build wayvid
print_header "Building wayvid"
cargo build --release
print_info "Build complete"
echo ""

# Kill any running instances
pkill -9 wayvid 2>/dev/null || true
sleep 1

# Test configurations
declare -A TESTS=(
    ["Hable (Default)"]="hable:1.0:hybrid"
    ["Hable (Cinema)"]="hable:1.2:rgb"
    ["Mobius (Detail)"]="mobius:0.3:hybrid"
    ["Mobius (Animation)"]="mobius:0.35:luma"
    ["Reinhard (Fast)"]="reinhard:0.5:hybrid"
    ["BT.2390 (Standard)"]="bt2390:1.0:auto"
)

# Function to create temp config
create_config() {
    local algo="$1"
    local param="$2"
    local mode="$3"
    local config_file="/tmp/wayvid-hdr-test-${algo}.yaml"
    
    cat > "$config_file" << EOF
source:
  type: file
  path: $VIDEO_PATH

hdr_mode: auto

tone_mapping:
  algorithm: $algo
  param: $param
  compute_peak: true
  mode: $mode

loop: false
mute: true
EOF
    
    echo "$config_file"
}

# Run tests
print_header "Running Tone Mapping Tests"
echo ""

TEST_NUM=0
TOTAL_TESTS=${#TESTS[@]}

for test_name in "${!TESTS[@]}"; do
    TEST_NUM=$((TEST_NUM + 1))
    IFS=':' read -r algo param mode <<< "${TESTS[$test_name]}"
    
    print_header "Test $TEST_NUM/$TOTAL_TESTS: $test_name"
    print_info "Algorithm: $algo, Param: $param, Mode: $mode"
    
    # Create config
    config_file=$(create_config "$algo" "$param" "$mode")
    
    # Run wayvid
    print_info "Starting playback..."
    timeout ${DURATION}s ./target/release/wayvid run \
        --config "$config_file" \
        --log-level "$LOG_LEVEL" \
        2>&1 | grep -E "(HDR|tone|mapping|Algorithm|Parameter|Mode)" || true
    
    # Kill wayvid
    pkill -9 wayvid 2>/dev/null || true
    sleep 1
    
    # Clean up
    rm -f "$config_file"
    
    print_info "Test complete"
    echo ""
    
    if [ $TEST_NUM -lt $TOTAL_TESTS ]; then
        print_warn "Next test in 2 seconds..."
        sleep 2
    fi
done

print_header "All Tests Complete"
print_info "Results:"
echo ""
echo "You should have seen different visual characteristics:"
echo "  • Hable: Good overall contrast and detail"
echo "  • Mobius: Softer highlights, more detail"
echo "  • Reinhard: Simple, fast"
echo "  • BT.2390: Natural, reference quality"
echo ""
print_info "Check the logs above for detailed HDR detection info"
print_info "Use '--log-level debug' for more detailed output"
