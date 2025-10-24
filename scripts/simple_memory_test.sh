#!/bin/bash
# Simple memory test - just test current branch
# Usage: ./simple_memory_test.sh [duration] [config]

set -e

DURATION=${1:-60}
CONFIG=${2:-test-config.yaml}
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=========================================="
echo "Simple Memory Test"
echo -e "==========================================${NC}"
echo "Duration: ${DURATION}s"
echo "Config: $CONFIG"
echo "Branch: $(git branch --show-current)"
echo ""

# Build
echo "🔨 Building..."
cargo build --release --all-features --quiet
echo -e "${GREEN}✓ Build complete${NC}"

# Start wayvid
echo -e "\n▶️  Starting wayvid..."
RUST_LOG=info "$PROJECT_DIR/target/release/wayvid" run -c "$CONFIG" >/dev/null 2>&1 &
PID=$!

# Wait for startup
sleep 3

if ! kill -0 $PID 2>/dev/null; then
    echo -e "${RED}❌ Failed to start${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Started (PID: $PID)${NC}"

# Monitor memory
echo -e "\n📊 Monitoring memory..."
INITIAL_RSS=$(awk '/VmRSS:/ {print $2}' /proc/$PID/status)
PEAK_RSS=$INITIAL_RSS
TOTAL_RSS=0
SAMPLES=0

echo "Time(s)  RSS(MB)  Peak(MB)"
echo "----------------------------"

for ((i=0; i<DURATION; i++)); do
    if ! kill -0 $PID 2>/dev/null; then
        echo -e "\n${RED}❌ Process died${NC}"
        exit 1
    fi
    
    RSS=$(awk '/VmRSS:/ {print $2}' /proc/$PID/status 2>/dev/null || echo "0")
    if [ "$RSS" -gt "$PEAK_RSS" ]; then
        PEAK_RSS=$RSS
    fi
    TOTAL_RSS=$((TOTAL_RSS + RSS))
    SAMPLES=$((SAMPLES + 1))
    
    # Print every 10 seconds
    if [ $((i % 10)) -eq 0 ]; then
        RSS_MB=$(awk "BEGIN {printf \"%.1f\", $RSS / 1024}")
        PEAK_MB=$(awk "BEGIN {printf \"%.1f\", $PEAK_RSS / 1024}")
        printf "%6d   %6s   %7s\n" $i "$RSS_MB" "$PEAK_MB"
    fi
    
    sleep 1
done

# Final stats
AVG_RSS=$((TOTAL_RSS / SAMPLES))
AVG_MB=$(awk "BEGIN {printf \"%.1f\", $AVG_RSS / 1024}")
PEAK_MB=$(awk "BEGIN {printf \"%.1f\", $PEAK_RSS / 1024}")
INITIAL_MB=$(awk "BEGIN {printf \"%.1f\", $INITIAL_RSS / 1024}")

# Stop wayvid
echo -e "\n🛑 Stopping wayvid..."
kill $PID 2>/dev/null
wait $PID 2>/dev/null

# Results
echo -e "\n${GREEN}=========================================="
echo "Results"
echo -e "==========================================${NC}"
echo "Initial RSS:  $INITIAL_MB MB"
echo "Average RSS:  $AVG_MB MB"
echo "Peak RSS:     $PEAK_MB MB"

# Growth
GROWTH=$(awk "BEGIN {printf \"%.1f\", ($PEAK_RSS - $INITIAL_RSS) / $INITIAL_RSS * 100}")
echo "Growth:       $GROWTH%"

# Check stability
if (( $(echo "$GROWTH < 10" | bc -l) )); then
    echo -e "${GREEN}✓ Memory stable (< 10% growth)${NC}"
elif (( $(echo "$GROWTH < 20" | bc -l) )); then
    echo -e "${YELLOW}⚠ Moderate growth (10-20%)${NC}"
else
    echo -e "${RED}❌ High growth (> 20%)${NC}"
fi
