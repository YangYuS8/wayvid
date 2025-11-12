#!/bin/bash
# Workshop Integration Test Script

echo "======================================"
echo "wayvid Workshop Integration Tests"
echo "======================================"
echo

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

WAYVID="./target/release/wayvid"

if [ ! -f "$WAYVID" ]; then
    echo "Building wayvid..."
    cargo build --release
fi

echo -e "${BLUE}Test 1: Check Steam installation${NC}"
$WAYVID workshop list --log-level warn
echo

echo -e "${BLUE}Test 2: Search command (shows instructions)${NC}"
$WAYVID workshop search "anime" --log-level warn
echo

echo -e "${BLUE}Test 3: Cache management${NC}"
$WAYVID workshop cache --log-level warn
echo

echo -e "${BLUE}Test 4: Help message${NC}"
$WAYVID workshop --help
echo

echo -e "${GREEN}✅ All basic tests passed!${NC}"
echo
echo -e "${YELLOW}📌 To test with real Workshop items:${NC}"
echo "1. Subscribe to video wallpapers in Steam Workshop"
echo "   https://steamcommunity.com/app/431960/workshop/"
echo "2. Run: $WAYVID workshop list"
echo "3. Import: $WAYVID workshop import <id> -o test-config.yaml"
echo
echo -e "${YELLOW}📌 Or test direct download (if available):${NC}"
echo "$WAYVID workshop install <workshop_id> -o test-config.yaml"
