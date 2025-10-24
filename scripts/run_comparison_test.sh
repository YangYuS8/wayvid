#!/bin/bash

# Issue #14: Memory Optimization Comparison Test
# Automatically compares baseline (main) vs optimized (m5-memory-opt)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/test_results"
CURRENT_BRANCH=$(git branch --show-current)

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================="
echo "Memory Optimization Comparison Test"
echo -e "==========================================${NC}"

# Test duration (default: 60 seconds for quick test)
DURATION=${1:-60}
CONFIG=${2:-test-config.yaml}

echo -e "\n${YELLOW}📋 Test Configuration:${NC}"
echo "  Duration: ${DURATION}s per test"
echo "  Duration: ${DURATION}s per test"
echo "  Config: $CONFIG"
echo "  Current branch: $CURRENT_BRANCH"

if [ ! -f "$PROJECT_ROOT/$CONFIG" ]; then
    echo -e "${RED}❌ Error: Config file not found: $CONFIG${NC}"
    exit 1
fi

mkdir -p "$RESULTS_DIR"

# Simple memory test function (works on any branch)
run_memory_test() {
    local test_duration=$1
    local config_file=$2
    local output_log=$3
    
    # Ensure output directory exists
    mkdir -p "$(dirname "$output_log")"
    
    # Start wayvid
    RUST_LOG=info "$PROJECT_ROOT/target/release/wayvid" run -c "$config_file" > "$output_log" 2>&1 &
    local pid=$!
    
    # Wait for startup
    sleep 3
    
    # Check if started
    if ! kill -0 $pid 2>/dev/null; then
        echo -e "${RED}❌ Failed to start${NC}"
        return 1
    fi
    
    # Collect memory stats
    local initial_rss=$(awk '/VmRSS:/ {print $2}' /proc/$pid/status)
    local peak_rss=$initial_rss
    local total_rss=0
    local samples=0
    
    echo "  Monitoring memory for ${test_duration}s..."
    for ((i=0; i<test_duration; i++)); do
        if ! kill -0 $pid 2>/dev/null; then
            echo -e "${RED}❌ Process died${NC}"
            return 1
        fi
        
        local rss=$(awk '/VmRSS:/ {print $2}' /proc/$pid/status 2>/dev/null || echo "0")
        if [ "$rss" -gt "$peak_rss" ]; then
            peak_rss=$rss
        fi
        total_rss=$((total_rss + rss))
        samples=$((samples + 1))
        
        sleep 1
    done
    
    # Calculate average
    local avg_rss=$((total_rss / samples))
    
    # Stop wayvid
    kill $pid 2>/dev/null
    wait $pid 2>/dev/null
    
    # Convert to MB and output
    local avg_mb=$(awk "BEGIN {printf \"%.1f\", $avg_rss / 1024}")
    local peak_mb=$(awk "BEGIN {printf \"%.1f\", $peak_rss / 1024}")
    
    echo "  Average RSS: $avg_mb MB" >> "$output_log"
    echo "  Peak RSS: $peak_mb MB" >> "$output_log"
    
    echo -e "${GREEN}✓ Complete${NC}"
    echo "  Average RSS: $avg_mb MB"
    echo "  Peak RSS: $peak_mb MB"
}

echo -e "\n${BLUE}==========================================${NC}"
echo -e "${YELLOW}Phase 1: Baseline Test (main branch)${NC}"
echo -e "${BLUE}==========================================${NC}"

# Save current work
echo "📦 Saving current branch state..."
git stash push -m "Temp: Comparison test" >/dev/null 2>&1 || true

# Switch to main
echo "🔄 Switching to main branch..."
git checkout main

# Build baseline
echo "🔨 Building baseline..."
cargo build --release --all-features >/dev/null 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Baseline build failed${NC}"
    git checkout "$CURRENT_BRANCH"
    git stash pop >/dev/null 2>&1 || true
    exit 1
fi

# Run baseline test
echo -e "${GREEN}▶️  Running baseline test (${DURATION}s)...${NC}"
BASELINE_LOG="$RESULTS_DIR/baseline_$(date +%Y%m%d_%H%M%S).log"
run_memory_test "$DURATION" "$CONFIG" "$BASELINE_LOG"

# Extract baseline stats
BASELINE_AVG=$(grep "Average RSS:" "$BASELINE_LOG" | awk '{print $3}')
BASELINE_PEAK=$(grep "Peak RSS:" "$BASELINE_LOG" | awk '{print $3}')

echo -e "\n${BLUE}==========================================${NC}"
echo -e "${YELLOW}Phase 2: Optimized Test (m5-memory-opt)${NC}"
echo -e "${BLUE}==========================================${NC}"

# Switch back
echo "🔄 Switching back to $CURRENT_BRANCH..."
git checkout "$CURRENT_BRANCH"
git stash pop >/dev/null 2>&1 || true

# Build optimized
echo "🔨 Building optimized version..."
cargo build --release --all-features >/dev/null 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Optimized build failed${NC}"
    exit 1
fi

# Run optimized test
echo -e "${GREEN}▶️  Running optimized test (${DURATION}s)...${NC}"
OPTIMIZED_LOG="$RESULTS_DIR/optimized_$(date +%Y%m%d_%H%M%S).log"
run_memory_test "$DURATION" "$CONFIG" "$OPTIMIZED_LOG"

# Extract optimized stats
OPTIMIZED_AVG=$(grep "Average RSS:" "$OPTIMIZED_LOG" | awk '{print $3}')
OPTIMIZED_PEAK=$(grep "Peak RSS:" "$OPTIMIZED_LOG" | awk '{print $3}')

echo -e "\n${BLUE}=========================================="
echo "📊 Comparison Results"
echo -e "==========================================${NC}"

# Calculate reduction
if [ -n "$BASELINE_AVG" ] && [ -n "$OPTIMIZED_AVG" ]; then
    AVG_REDUCTION=$(awk "BEGIN {printf \"%.1f\", (1 - $OPTIMIZED_AVG / $BASELINE_AVG) * 100}")
    PEAK_REDUCTION=$(awk "BEGIN {printf \"%.1f\", (1 - $OPTIMIZED_PEAK / $BASELINE_PEAK) * 100}")
    
    echo -e "\n${YELLOW}Average RSS:${NC}"
    echo "  Baseline:  ${BASELINE_AVG} MB"
    echo "  Optimized: ${OPTIMIZED_AVG} MB"
    echo -e "  ${GREEN}Reduction: ${AVG_REDUCTION}%${NC}"
    
    echo -e "\n${YELLOW}Peak RSS:${NC}"
    echo "  Baseline:  ${BASELINE_PEAK} MB"
    echo "  Optimized: ${OPTIMIZED_PEAK} MB"
    echo -e "  ${GREEN}Reduction: ${PEAK_REDUCTION}%${NC}"
    
    # Check success criteria
    echo -e "\n${YELLOW}Success Criteria:${NC}"
    
    # Target: >50% reduction (ideally >73%)
    if (( $(echo "$AVG_REDUCTION > 73" | bc -l) )); then
        echo -e "  ${GREEN}✅ Target achieved! (>${AVG_REDUCTION}% > 73%)${NC}"
    elif (( $(echo "$AVG_REDUCTION > 50" | bc -l) )); then
        echo -e "  ${GREEN}✅ Minimum achieved (>${AVG_REDUCTION}% > 50%)${NC}"
        echo -e "  ${YELLOW}⚠️  Below ideal target of 73%${NC}"
    else
        echo -e "  ${RED}❌ Below minimum target (${AVG_REDUCTION}% < 50%)${NC}"
    fi
    
    # Extract pressure events
    PRESSURE_EVENTS=$(grep -c "Memory pressure" "$OPTIMIZED_LOG" 2>/dev/null || echo "0")
    echo -e "  Memory pressure events: $PRESSURE_EVENTS"
    
    # Extract decoder sharing info
    SHARED_DECODERS=$(grep -c "Reusing existing decoder" "$OPTIMIZED_LOG" 2>/dev/null || echo "0")
    echo -e "  Decoder sharing count: $SHARED_DECODERS"
else
    echo -e "${RED}❌ Could not extract statistics${NC}"
fi

echo -e "\n${BLUE}==========================================${NC}"
echo -e "${YELLOW}📁 Results saved to:${NC}"
echo "  Baseline:  $BASELINE_LOG"
echo "  Optimized: $OPTIMIZED_LOG"
echo -e "${BLUE}==========================================${NC}"
