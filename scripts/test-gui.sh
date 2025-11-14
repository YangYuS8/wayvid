#!/bin/bash
# GUI Integration Test Script
# 测试 GUI 与 daemon 的集成

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WAYVID_BIN="$PROJECT_ROOT/target/release/wayvid"
WAYVID_GUI="$PROJECT_ROOT/target/release/wayvid-gui"
WAYVID_CTL="$PROJECT_ROOT/target/release/wayvid-ctl"
TEST_CONFIG="$PROJECT_ROOT/examples/simple-config.yaml"
LOG_FILE="/tmp/wayvid-gui-test.log"

echo "🧪 wayvid GUI Integration Test"
echo "================================"
echo ""

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# 1. Check binaries
echo "1️⃣  Checking binaries..."
if [ ! -f "$WAYVID_BIN" ]; then
    error "wayvid binary not found. Run: cargo build --release"
    exit 1
fi
success "wayvid binary found"

if [ ! -f "$WAYVID_GUI" ]; then
    error "wayvid-gui binary not found. Run: cargo build --release --bin wayvid-gui --features gui"
    exit 1
fi
success "wayvid-gui binary found"

if [ ! -f "$WAYVID_CTL" ]; then
    error "wayvid-ctl binary not found. Run: cargo build --release --bin wayvid-ctl"
    exit 1
fi
success "wayvid-ctl binary found"
echo ""

# 2. Check daemon
echo "2️⃣  Checking daemon status..."
if pgrep -f "wayvid run" > /dev/null; then
    success "wayvid daemon is running"
    DAEMON_PID=$(pgrep -f "wayvid run")
    info "PID: $DAEMON_PID"
else
    warning "wayvid daemon not running, starting..."
    
    if [ ! -f "$TEST_CONFIG" ]; then
        error "Config file not found: $TEST_CONFIG"
        exit 1
    fi
    
    # Start daemon in background
    "$WAYVID_BIN" run --config "$TEST_CONFIG" > "$LOG_FILE" 2>&1 &
    DAEMON_PID=$!
    
    # Wait for daemon to start
    sleep 2
    
    if pgrep -f "wayvid run" > /dev/null; then
        success "Daemon started successfully (PID: $DAEMON_PID)"
    else
        error "Failed to start daemon. Check logs: $LOG_FILE"
        exit 1
    fi
fi
echo ""

# 3. Check IPC socket
echo "3️⃣  Checking IPC socket..."
SOCKET_PATH="/run/user/$(id -u)/wayvid.sock"
if [ -S "$SOCKET_PATH" ]; then
    success "IPC socket exists: $SOCKET_PATH"
else
    error "IPC socket not found: $SOCKET_PATH"
    exit 1
fi
echo ""

# 4. Test CLI control (optional)
echo "4️⃣  Testing CLI control..."
if "$WAYVID_CTL" status > /dev/null 2>&1; then
    success "CLI control working"
    
    # Get output info
    OUTPUT_INFO=$("$WAYVID_CTL" status 2>&1 | head -10)
    info "Output status:"
    echo "$OUTPUT_INFO" | sed 's/^/    /'
else
    warning "CLI control not responding (might be normal)"
fi
echo ""

# 5. GUI test instructions
echo "5️⃣  GUI Manual Test Checklist:"
echo "================================"
info "Starting GUI in 3 seconds..."
sleep 3

echo ""
echo "📋 Test Checklist:"
echo ""
echo "   [ ] 1. GUI window opens successfully"
echo "   [ ] 2. Connection status shows '● Connected'"
echo "   [ ] 3. Click '📡 Connect' if not connected"
echo ""
echo "   📺 Outputs Tab:"
echo "   [ ] 4. Outputs are listed (check for eDP-1 or similar)"
echo "   [ ] 5. Select an output (click checkbox)"
echo "   [ ] 6. Click 'Pause' button"
echo "   [ ] 7. Click 'Resume' button"
echo ""
echo "   🎬 Video Sources Tab:"
echo "   [ ] 8. Enter a video path: ~/Videos/wallpaper.mp4"
echo "   [ ] 9. Click 'Quick Access' buttons"
echo "   [ ] 10. Click '✓ Apply to Selected Output'"
echo ""
echo "   🎮 Workshop Tab:"
echo "   [ ] 11. Click '🔄 Scan Workshop'"
echo "   [ ] 12. Search for items (if any found)"
echo "   [ ] 13. Click 'Preview' on an item"
echo "   [ ] 14. Click 'Import' on an item"
echo ""
echo "   ⚙ Settings Tab:"
echo "   [ ] 15. Change Layout Mode dropdown"
echo "   [ ] 16. Adjust volume slider"
echo "   [ ] 17. Toggle loop/mute checkboxes"
echo "   [ ] 18. Click '💾 Apply to Selected Output'"
echo ""
echo "   🔄 General:"
echo "   [ ] 19. No crashes or freezes"
echo "   [ ] 20. UI is responsive"
echo "   [ ] 21. Close GUI with Ctrl+Q or window close"
echo ""

# Start GUI
info "Launching GUI..."
"$WAYVID_GUI"

# After GUI closes
echo ""
echo "================================"
echo "🏁 GUI Test Complete"
echo ""
echo "Check logs if needed:"
echo "  Daemon: $LOG_FILE"
echo "  Daemon output: tail -f $LOG_FILE"
echo ""
echo "To stop daemon:"
echo "  kill $DAEMON_PID"
echo ""

success "Test finished"
