#!/bin/bash
# Memory usage testing script for M5 Issue #14
# Tests memory consumption before and after optimization

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TEST_DURATION=${1:-60}  # Default 60 seconds
CONFIG_FILE="${2:-$PROJECT_DIR/test-config.yaml}"

echo "=========================================="
echo "Memory Usage Test for Issue #14"
echo "=========================================="
echo "Duration: ${TEST_DURATION}s"
echo "Config: $CONFIG_FILE"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if config file exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${RED}Error: Config file not found: $CONFIG_FILE${NC}"
    exit 1
fi

# Build the project
echo -e "${BLUE}📦 Building project...${NC}"
cd "$PROJECT_DIR"
cargo build --release --all-features 2>&1 | grep -E "(Finished|Compiling wayvid)" || true
echo ""

# Function to measure memory usage
measure_memory() {
    local pid=$1
    local duration=$2
    local output_file=$3
    
    echo "timestamp,vss_kb,rss_kb,shared_kb" > "$output_file"
    
    local end_time=$((SECONDS + duration))
    while [ $SECONDS -lt $end_time ]; do
        if ! kill -0 $pid 2>/dev/null; then
            echo -e "${RED}Process died unexpectedly${NC}"
            return 1
        fi
        
        # Get memory stats from /proc/[pid]/status
        local mem_info=$(grep -E "^(VmSize|VmRSS|RssFile):" /proc/$pid/status 2>/dev/null || echo "")
        
        if [ -n "$mem_info" ]; then
            local vss=$(echo "$mem_info" | grep VmSize | awk '{print $2}')
            local rss=$(echo "$mem_info" | grep VmRSS | awk '{print $2}')
            local shared=$(echo "$mem_info" | grep RssFile | awk '{print $2}')
            
            echo "$SECONDS,$vss,$rss,$shared" >> "$output_file"
        fi
        
        sleep 1
    done
    
    return 0
}

# Function to calculate statistics
calculate_stats() {
    local data_file=$1
    
    # Skip header, calculate avg, min, max
    awk -F',' '
    NR > 1 {
        vss_sum += $2; rss_sum += $3; shared_sum += $4;
        if (NR == 2 || $2 < vss_min) vss_min = $2;
        if (NR == 2 || $3 < rss_min) rss_min = $3;
        if (NR == 2 || $2 > vss_max) vss_max = $2;
        if (NR == 2 || $3 > rss_max) rss_max = $3;
        count++;
    }
    END {
        if (count > 0) {
            printf "VSS: avg=%.0f min=%.0f max=%.0f KB\n", vss_sum/count, vss_min, vss_max;
            printf "RSS: avg=%.0f min=%.0f max=%.0f KB\n", rss_sum/count, rss_min, rss_max;
            printf "RSS (MB): avg=%.1f min=%.1f max=%.1f\n", rss_sum/count/1024, rss_min/1024, rss_max/1024;
        }
    }' "$data_file"
}

# Run the test
echo -e "${BLUE}🧪 Starting memory test...${NC}"
echo ""

OUTPUT_DIR="$PROJECT_DIR/test_results"
mkdir -p "$OUTPUT_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
MEMORY_LOG="$OUTPUT_DIR/memory_${TIMESTAMP}.csv"
APP_LOG="$OUTPUT_DIR/app_${TIMESTAMP}.log"

# Start wayvid in background
echo -e "${YELLOW}▶️  Starting wayvid...${NC}"
RUST_LOG=debug "$PROJECT_DIR/target/release/wayvid" run -c "$CONFIG_FILE" > "$APP_LOG" 2>&1 &
WAYVID_PID=$!

# Wait for startup
sleep 3

# Check if process is still running
if ! kill -0 $WAYVID_PID 2>/dev/null; then
    echo -e "${RED}❌ wayvid failed to start!${NC}"
    echo ""
    echo "Last 20 lines of log:"
    tail -20 "$APP_LOG"
    exit 1
fi

echo -e "${GREEN}✓ wayvid started (PID: $WAYVID_PID)${NC}"
echo ""

# Measure memory usage
echo -e "${BLUE}📊 Measuring memory for ${TEST_DURATION}s...${NC}"
if measure_memory $WAYVID_PID $TEST_DURATION "$MEMORY_LOG"; then
    echo -e "${GREEN}✓ Measurement complete${NC}"
else
    echo -e "${RED}✗ Measurement failed${NC}"
    kill $WAYVID_PID 2>/dev/null || true
    exit 1
fi

# Stop wayvid
echo ""
echo -e "${YELLOW}⏹️  Stopping wayvid...${NC}"
kill -TERM $WAYVID_PID 2>/dev/null || true
wait $WAYVID_PID 2>/dev/null || true
echo -e "${GREEN}✓ Stopped${NC}"
echo ""

# Analyze results
echo "=========================================="
echo -e "${BLUE}📈 Memory Statistics${NC}"
echo "=========================================="
calculate_stats "$MEMORY_LOG"
echo ""

# Check for memory leaks
echo "=========================================="
echo -e "${BLUE}🔍 Leak Analysis${NC}"
echo "=========================================="

# Get first and last RSS values
FIRST_RSS=$(awk -F',' 'NR==2 {print $3}' "$MEMORY_LOG")
LAST_RSS=$(tail -1 "$MEMORY_LOG" | cut -d',' -f3)

if [ -n "$FIRST_RSS" ] && [ -n "$LAST_RSS" ]; then
    GROWTH=$(echo "$LAST_RSS - $FIRST_RSS" | bc)
    GROWTH_PERCENT=$(echo "scale=2; ($GROWTH / $FIRST_RSS) * 100" | bc)
    
    echo "Initial RSS: $(echo "scale=1; $FIRST_RSS / 1024" | bc) MB"
    echo "Final RSS:   $(echo "scale=1; $LAST_RSS / 1024" | bc) MB"
    echo "Growth:      $(echo "scale=1; $GROWTH / 1024" | bc) MB ($GROWTH_PERCENT%)"
    echo ""
    
    if (( $(echo "$GROWTH_PERCENT > 10" | bc -l) )); then
        echo -e "${YELLOW}⚠️  Warning: Memory grew by more than 10%${NC}"
    else
        echo -e "${GREEN}✓ Memory stable (growth < 10%)${NC}"
    fi
else
    echo -e "${RED}Could not analyze leak${NC}"
fi
echo ""

# Check for decoder sharing in logs
echo "=========================================="
echo -e "${BLUE}🔄 Decoder Sharing Analysis${NC}"
echo "=========================================="

DECODER_CREATED=$(grep -c "Creating new shared decoder" "$APP_LOG" || echo "0")
DECODER_REUSED=$(grep -c "Reusing existing decoder" "$APP_LOG" || echo "0")

echo "Decoders created: $DECODER_CREATED"
echo "Decoders reused:  $DECODER_REUSED"

if [ "$DECODER_REUSED" -gt 0 ]; then
    echo -e "${GREEN}✓ Decoder sharing is working${NC}"
else
    echo -e "${YELLOW}⚠️  No decoder reuse detected${NC}"
fi
echo ""

# Check for memory pressure events
echo "=========================================="
echo -e "${BLUE}💾 Memory Pressure Events${NC}"
echo "=========================================="

PRESSURE_HIGH=$(grep -c "High memory pressure" "$APP_LOG" || echo "0")
PRESSURE_CRITICAL=$(grep -c "Critical memory pressure" "$APP_LOG" || echo "0")
POOL_CLEARED=$(grep -c "clearing buffer pool" "$APP_LOG" || echo "0")

echo "High pressure events:     $PRESSURE_HIGH"
echo "Critical pressure events: $PRESSURE_CRITICAL"
echo "Pool clears:              $POOL_CLEARED"

if [ "$PRESSURE_CRITICAL" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Critical pressure detected - may need tuning${NC}"
elif [ "$PRESSURE_HIGH" -gt 0 ]; then
    echo -e "${YELLOW}ℹ️  Some high pressure - system is managing memory${NC}"
else
    echo -e "${GREEN}✓ No significant memory pressure${NC}"
fi
echo ""

# Summary
echo "=========================================="
echo -e "${BLUE}📋 Test Summary${NC}"
echo "=========================================="
echo "Memory log: $MEMORY_LOG"
echo "App log:    $APP_LOG"
echo ""
echo -e "${GREEN}✓ Memory test complete!${NC}"
echo ""

# Compare to baseline if exists
BASELINE_FILE="$OUTPUT_DIR/baseline_memory.txt"
if [ -f "$BASELINE_FILE" ]; then
    echo "=========================================="
    echo -e "${BLUE}📊 Comparison to Baseline${NC}"
    echo "=========================================="
    
    BASELINE_AVG=$(grep "RSS (MB): avg=" "$BASELINE_FILE" | cut -d'=' -f2 | cut -d' ' -f1)
    CURRENT_AVG=$(calculate_stats "$MEMORY_LOG" | grep "RSS (MB): avg=" | cut -d'=' -f2 | cut -d' ' -f1)
    
    if [ -n "$BASELINE_AVG" ] && [ -n "$CURRENT_AVG" ]; then
        REDUCTION=$(echo "scale=1; $BASELINE_AVG - $CURRENT_AVG" | bc)
        REDUCTION_PERCENT=$(echo "scale=1; ($REDUCTION / $BASELINE_AVG) * 100" | bc)
        
        echo "Baseline:    ${BASELINE_AVG} MB"
        echo "Current:     ${CURRENT_AVG} MB"
        echo "Reduction:   ${REDUCTION} MB ($REDUCTION_PERCENT%)"
        echo ""
        
        if (( $(echo "$REDUCTION_PERCENT > 50" | bc -l) )); then
            echo -e "${GREEN}🎉 Excellent! >50% reduction achieved!${NC}"
        elif (( $(echo "$REDUCTION_PERCENT > 30" | bc -l) )); then
            echo -e "${GREEN}✓ Good progress (>30% reduction)${NC}"
        elif (( $(echo "$REDUCTION_PERCENT > 0" | bc -l) )); then
            echo -e "${YELLOW}⚠️  Some improvement but below target${NC}"
        else
            echo -e "${RED}❌ Memory usage increased${NC}"
        fi
    fi
    echo ""
else
    echo "Tip: Save current results as baseline with:"
    echo "  calculate_stats \"$MEMORY_LOG\" > \"$BASELINE_FILE\""
    echo ""
fi

exit 0
