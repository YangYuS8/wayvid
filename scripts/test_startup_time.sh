#!/bin/bash
# Test script to measure wayvid startup time
# Tests both baseline (main) and optimized (m5-lazy-init) branches

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_RESULTS_DIR="$PROJECT_DIR/test_results"
CONFIG_FILE="$TEST_RESULTS_DIR/test-config.yaml"

# Ensure test results directory exists
mkdir -p "$TEST_RESULTS_DIR"

# Create simple test config
cat > "$CONFIG_FILE" << 'EOF'
source: /home/yangyus8/code/edupal/功能演示.mp4
layout: fill
hwdec: auto
loop: true
mute: true
EOF

echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo -e "${BLUE}   wayvid Startup Time Test (Issue #15)    ${NC}"
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo ""

# Function to measure startup time
measure_startup() {
    local branch=$1
    local log_file=$2
    
    echo -e "${BLUE}📊 Testing branch: ${branch}${NC}"
    
    # Build the binary
    echo "  Building..."
    cd "$PROJECT_DIR"
    cargo build --release --all-features > /dev/null 2>&1
    
    # Run wayvid and capture log
    echo "  Starting wayvid..."
    timeout 5 RUST_LOG=info "$PROJECT_DIR/target/release/wayvid" run -c "$CONFIG_FILE" > "$log_file" 2>&1 &
    WAYVID_PID=$!
    
    # Wait a bit for startup
    sleep 3
    
    # Kill process
    kill $WAYVID_PID 2>/dev/null || true
    wait $WAYVID_PID 2>/dev/null || true
    
    # Extract startup time
    if grep -q "Startup complete" "$log_file"; then
        local startup_time=$(grep "Startup complete" "$log_file" | sed -n 's/.*in \([0-9.]*\)ms.*/\1/p')
        echo -e "  ${GREEN}✓ Startup time: ${startup_time}ms${NC}"
        echo "$startup_time"
    else
        echo -e "  ${RED}✗ Could not find startup time in log${NC}"
        echo "0"
    fi
}

# Current branch (should be m5-lazy-init)
CURRENT_BRANCH=$(git branch --show-current)

# Test optimized branch (current)
echo -e "${YELLOW}Testing optimized branch (${CURRENT_BRANCH})...${NC}"
echo ""
OPTIMIZED_LOG="$TEST_RESULTS_DIR/startup_optimized_$(date +%Y%m%d_%H%M%S).log"
OPTIMIZED_TIME=$(measure_startup "$CURRENT_BRANCH" "$OPTIMIZED_LOG")
echo ""

# Switch to main branch for baseline
echo -e "${YELLOW}Testing baseline branch (main)...${NC}"
echo ""
git stash -q
git checkout main -q
BASELINE_LOG="$TEST_RESULTS_DIR/startup_baseline_$(date +%Y%m%d_%H%M%S).log"
BASELINE_TIME=$(measure_startup "main" "$BASELINE_LOG")
echo ""

# Return to original branch
git checkout "$CURRENT_BRANCH" -q
if git stash list | grep -q "stash@{0}"; then
    git stash pop -q
fi

# Calculate improvement
if [ "$BASELINE_TIME" != "0" ] && [ "$OPTIMIZED_TIME" != "0" ]; then
    IMPROVEMENT=$(echo "scale=1; (($BASELINE_TIME - $OPTIMIZED_TIME) / $BASELINE_TIME) * 100" | bc)
    
    echo -e "${BLUE}════════════════════════════════════════════${NC}"
    echo -e "${GREEN}   Results Summary${NC}"
    echo -e "${BLUE}════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  Baseline (main):    ${BASELINE_TIME}ms"
    echo -e "  Optimized (${CURRENT_BRANCH}): ${OPTIMIZED_TIME}ms"
    echo -e ""
    echo -e "  ${GREEN}Improvement: ${IMPROVEMENT}%${NC}"
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════${NC}"
    echo ""
    echo "Logs saved to:"
    echo "  Baseline:  $BASELINE_LOG"
    echo "  Optimized: $OPTIMIZED_LOG"
else
    echo -e "${RED}Failed to measure startup times${NC}"
    exit 1
fi
