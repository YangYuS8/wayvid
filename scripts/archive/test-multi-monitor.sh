#!/bin/bash
# Multi-Monitor Testing Script for Issue #2

set -e

echo "==================================================="
echo "Multi-Monitor Testing for Issue #2"
echo "==================================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

WAYVID="./target/release/wayvid"
WAYVID_CTL="./target/release/wayvid-ctl"
CONFIG="./test-multi-monitor-config.yaml"

# Check if binaries exist
if [ ! -f "$WAYVID" ]; then
    echo -e "${RED}Error: wayvid binary not found. Run 'cargo build --release' first.${NC}"
    exit 1
fi

if [ ! -f "$WAYVID_CTL" ]; then
    echo -e "${RED}Error: wayvid-ctl binary not found. Run 'cargo build --release' first.${NC}"
    exit 1
fi

# Kill any existing wayvid instance
echo -e "${YELLOW}[1/8] Cleaning up existing wayvid instances...${NC}"
pkill -9 wayvid 2>/dev/null || true
sleep 1

# Test 1: Pattern Matching Configuration
echo ""
echo -e "${BLUE}==================================================="
echo -e "Test 1: Pattern Matching with Priority"
echo -e "===================================================${NC}"
echo ""
echo "Expected behavior:"
echo "  - eDP-1 (laptop): RED video (exact match, priority 0)"
echo "  - HDMI-A-1: GREEN video (HDMI-A-* pattern, priority 5)"
echo ""
echo -e "${YELLOW}Starting wayvid with test config...${NC}"
$WAYVID --config "$CONFIG" &
WAYVID_PID=$!
sleep 3

echo -e "${GREEN}✓ wayvid started (PID: $WAYVID_PID)${NC}"
echo ""
echo -e "${YELLOW}Please verify:${NC}"
echo "  1. eDP-1 shows RED background"
echo "  2. HDMI-A-1 shows GREEN background"
echo "  3. Both are playing smoothly"
echo ""
read -p "Press Enter when verified..."

# Test 2: Get Status
echo ""
echo -e "${BLUE}==================================================="
echo -e "Test 2: IPC Status Command"
echo -e "===================================================${NC}"
echo ""
$WAYVID_CTL status | jq '.'
echo ""
read -p "Press Enter to continue..."

# Test 3: Dynamic Source Switching - File
echo ""
echo -e "${BLUE}==================================================="
echo -e "Test 3: Dynamic Source Switching (File)"
echo -e "===================================================${NC}"
echo ""
echo "Switching HDMI-A-1 to BLUE video (hdmi-generic.mp4)..."
$WAYVID_CTL switch -o HDMI-A-1 /home/yangyus8/Videos/hdmi-generic.mp4
sleep 2
echo ""
echo -e "${YELLOW}Please verify:${NC}"
echo "  - HDMI-A-1 now shows BLUE background"
echo ""
read -p "Press Enter when verified..."

# Test 4: Dynamic Source Switching - Another File
echo ""
echo -e "${BLUE}==================================================="
echo -e "Test 4: Dynamic Source Switching (Another File)"
echo -e "===================================================${NC}"
echo ""
echo "Switching eDP-1 to YELLOW video (fallback.mp4)..."
$WAYVID_CTL switch -o eDP-1 /home/yangyus8/Videos/fallback.mp4
sleep 2
echo ""
echo -e "${YELLOW}Please verify:${NC}"
echo "  - eDP-1 now shows YELLOW background"
echo ""
read -p "Press Enter when verified..."

# Test 5: Reload Config
echo ""
echo -e "${BLUE}==================================================="
echo -e "Test 5: Config Reload"
echo -e "===================================================${NC}"
echo ""
echo "Reloading configuration (should restore original sources)..."
$WAYVID_CTL reload
sleep 2
echo ""
echo -e "${YELLOW}Please verify:${NC}"
echo "  - eDP-1 back to RED background"
echo "  - HDMI-A-1 back to GREEN background"
echo ""
read -p "Press Enter when verified..."

# Test 6: Pause/Resume
echo ""
echo -e "${BLUE}==================================================="
echo -e "Test 6: Pause/Resume Control"
echo -e "===================================================${NC}"
echo ""
echo "Pausing HDMI-A-1..."
$WAYVID_CTL pause -o HDMI-A-1
sleep 1
echo -e "${YELLOW}Please verify: HDMI-A-1 is paused${NC}"
read -p "Press Enter to resume..."

echo "Resuming HDMI-A-1..."
$WAYVID_CTL resume -o HDMI-A-1
sleep 1
echo -e "${YELLOW}Please verify: HDMI-A-1 is playing again${NC}"
read -p "Press Enter to continue..."

# Test 7: Volume Control
echo ""
echo -e "${BLUE}==================================================="
echo -e "Test 7: Volume Control"
echo -e "===================================================${NC}"
echo ""
echo "Setting eDP-1 volume to 0.8..."
$WAYVID_CTL volume -o eDP-1 0.8
sleep 1
echo "Setting HDMI-A-1 volume to 0.2..."
$WAYVID_CTL volume -o HDMI-A-1 0.2
sleep 1
echo -e "${GREEN}✓ Volume commands sent${NC}"
read -p "Press Enter to continue..."

# Test 8: Hot-plug Simulation (if possible)
echo ""
echo -e "${BLUE}==================================================="
echo -e "Test 8: Hot-plug Handling (Manual)"
echo -e "===================================================${NC}"
echo ""
echo "If you have another monitor available:"
echo "  1. Disconnect HDMI-A-1"
echo "  2. Wait 3 seconds"
echo "  3. Reconnect HDMI-A-1"
echo ""
echo "Expected: Monitor should reconnect with GREEN video (HDMI-A-* pattern)"
echo ""
read -p "Press Enter when ready to continue (or skip if no hot-plug test)..."

# Final Status
echo ""
echo -e "${BLUE}==================================================="
echo -e "Final Status Check"
echo -e "===================================================${NC}"
echo ""
$WAYVID_CTL status | jq -r '.outputs[] | "Output: \(.name) - Source: \(.source.path // .source.url // "N/A")"'
echo ""

# Cleanup
echo ""
echo -e "${YELLOW}Cleaning up...${NC}"
$WAYVID_CTL quit || true
sleep 1
pkill -9 wayvid 2>/dev/null || true

echo ""
echo -e "${GREEN}==================================================="
echo -e "Testing Complete!"
echo -e "===================================================${NC}"
echo ""
echo "Summary of tests:"
echo "  ✓ Pattern matching with priority"
echo "  ✓ IPC status command"
echo "  ✓ Dynamic source switching (file)"
echo "  ✓ Config reload"
echo "  ✓ Pause/resume control"
echo "  ✓ Volume control"
echo "  ✓ Hot-plug handling (manual)"
echo ""
echo "If all tests passed, Issue #2 is ready to merge!"
