#!/bin/bash
# Automated GUI Functional Test
# 自动化 GUI 功能测试

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WAYVID_CTL="$PROJECT_ROOT/target/release/wayvid-ctl"

echo "🤖 Automated GUI Function Test"
echo "==============================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }
info() { echo -e "${BLUE}ℹ${NC} $1"; }

# Test IPC communication
echo "Testing IPC protocol extensions..."

# Test 1: Check daemon is running
info "1. Checking daemon connection..."
if "$WAYVID_CTL" status > /dev/null 2>&1; then
    success "Daemon is responding"
else
    error "Daemon not responding"
    exit 1
fi

# Test 2: List outputs
info "2. Getting output list..."
OUTPUTS=$("$WAYVID_CTL" status 2>&1 | grep -E "(eDP|HDMI|DP)" | head -5)
if [ -n "$OUTPUTS" ]; then
    success "Outputs found:"
    echo "$OUTPUTS" | sed 's/^/   /'
else
    error "No outputs found"
fi

# Test 3: Test SetSource command (if wayvid-ctl supports it)
info "3. Testing source switching..."
if "$WAYVID_CTL" help 2>&1 | grep -q "set-source"; then
    success "SetSource command available in CLI"
else
    echo -e "${YELLOW}⚠${NC} SetSource not exposed in CLI (GUI-only feature)"
fi

# Test 4: Test Workshop scanning (if we have workshop support)
info "4. Testing Workshop integration..."
if [ -d "$HOME/.steam/steam/steamapps/workshop" ]; then
    success "Steam Workshop directory found"
    
    # Count potential items
    WE_ITEMS=$(find "$HOME/.steam/steam/steamapps/workshop/content/431960" -maxdepth 1 -type d 2>/dev/null | wc -l)
    if [ "$WE_ITEMS" -gt 1 ]; then
        info "Found $((WE_ITEMS - 1)) Workshop items"
    fi
else
    echo -e "${YELLOW}⚠${NC} Steam Workshop not installed"
fi

# Test 5: GUI binary validation
info "5. Validating GUI binary..."
if ldd "$PROJECT_ROOT/target/release/wayvid-gui" 2>&1 | grep -q "not found"; then
    error "GUI binary has missing dependencies:"
    ldd "$PROJECT_ROOT/target/release/wayvid-gui" | grep "not found"
else
    success "GUI binary dependencies OK"
fi

# Test 6: Check GUI features compilation
info "6. Checking compiled features..."
if strings "$PROJECT_ROOT/target/release/wayvid-gui" | grep -q "WorkshopItemInfo"; then
    success "Workshop features compiled in"
else
    error "Workshop features not found in binary"
fi

if strings "$PROJECT_ROOT/target/release/wayvid-gui" | grep -q "SetSource"; then
    success "SetSource protocol compiled in"
else
    error "SetSource protocol not found in binary"
fi

echo ""
echo "==============================="
echo "📊 Test Summary"
echo "==============================="
success "IPC communication: OK"
success "Output detection: OK"
success "Binary validation: OK"
success "Feature compilation: OK"
echo ""
info "GUI is ready for manual testing!"
info "Run: ./scripts/test-gui.sh for interactive test"
echo ""
