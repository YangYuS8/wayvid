#!/usr/bin/env bash
#
# M5 Performance Testing Script
# Automatically measures CPU and memory usage for baseline vs m5-shared-decode
#
# Usage: ./scripts/test_m5_performance.sh [duration_seconds]
#

set -e

# Configuration
DURATION=${1:-60}  # Default 60 seconds
SAMPLE_INTERVAL=2  # Sample every 2 seconds
CONFIG_FILE="${HOME}/.config/wayvid/test-config.toml"
RESULTS_DIR="./test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Utility functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if in git repo
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "Not in a git repository!"
        exit 1
    fi
    
    # Check if config exists
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log_warning "Config file not found: $CONFIG_FILE"
        log_info "Creating default config..."
        mkdir -p "$(dirname "$CONFIG_FILE")"
        cat > "$CONFIG_FILE" << 'EOF'
[video]
source = { file = "/home/yangyus8/test.mp4" }
loop_playback = true

[video.hwdec]
mode = "auto"

[render]
layout = "contain"
EOF
        log_success "Created config: $CONFIG_FILE"
        log_warning "Please place a test video at: /home/yangyus8/test.mp4"
        exit 1
    fi
    
    # Check if test video exists
    local video_path=$(grep -oP 'file = "\K[^"]+' "$CONFIG_FILE" || echo "")
    if [[ -n "$video_path" ]] && [[ ! -f "$video_path" ]]; then
        log_error "Test video not found: $video_path"
        exit 1
    fi
    
    log_success "Prerequisites OK"
}

measure_performance() {
    local branch=$1
    local output_file=$2
    
    log_info "Measuring performance on branch: $branch"
    
    # Build release version
    log_info "Building release version..."
    cargo build --release --features video-mpv 2>&1 | grep -E "(Compiling|Finished)" || true
    
    # Start wayvid in background
    log_info "Starting wayvid..."
    RUST_LOG=wayvid::video::shared_decode=debug,info \
        ./target/release/wayvid \
        --config "$CONFIG_FILE" \
        --log-level info \
        > "${output_file}.log" 2>&1 &
    
    local wayvid_pid=$!
    log_info "Started wayvid (PID: $wayvid_pid)"
    
    # Wait for initialization
    sleep 5
    
    # Check if still running
    if ! kill -0 $wayvid_pid 2>/dev/null; then
        log_error "wayvid crashed during startup!"
        log_error "Check log: ${output_file}.log"
        return 1
    fi
    
    # Collect performance data
    log_info "Collecting data for ${DURATION} seconds..."
    echo "timestamp,cpu_percent,mem_mb,threads" > "$output_file"
    
    local samples=$((DURATION / SAMPLE_INTERVAL))
    for ((i=1; i<=samples; i++)); do
        # Get CPU and memory using ps
        local stats=$(ps -p $wayvid_pid -o %cpu,rss,nlwp --no-headers 2>/dev/null || echo "0 0 0")
        local cpu=$(echo $stats | awk '{print $1}')
        local mem_kb=$(echo $stats | awk '{print $2}')
        local threads=$(echo $stats | awk '{print $3}')
        local mem_mb=$(echo "scale=2; $mem_kb / 1024" | bc)
        local timestamp=$(date +%s)
        
        echo "${timestamp},${cpu},${mem_mb},${threads}" >> "$output_file"
        
        # Progress indicator
        local progress=$((i * 100 / samples))
        printf "\r  Progress: [%-50s] %d%%" $(printf '#%.0s' $(seq 1 $((progress / 2)))) $progress
        
        sleep $SAMPLE_INTERVAL
    done
    echo ""
    
    # Stop wayvid
    log_info "Stopping wayvid..."
    kill $wayvid_pid 2>/dev/null || true
    wait $wayvid_pid 2>/dev/null || true
    
    # Calculate statistics
    calculate_stats "$output_file"
    
    log_success "Performance data saved to: $output_file"
}

calculate_stats() {
    local data_file=$1
    
    # Skip header line and calculate averages
    local avg_cpu=$(awk -F',' 'NR>1 {sum+=$2; count++} END {if(count>0) print sum/count; else print 0}' "$data_file")
    local avg_mem=$(awk -F',' 'NR>1 {sum+=$3; count++} END {if(count>0) print sum/count; else print 0}' "$data_file")
    local avg_threads=$(awk -F',' 'NR>1 {sum+=$4; count++} END {if(count>0) print sum/count; else print 0}' "$data_file")
    
    # Find peak values
    local peak_cpu=$(awk -F',' 'NR>1 {if($2>max) max=$2} END {print max}' "$data_file")
    local peak_mem=$(awk -F',' 'NR>1 {if($3>max) max=$3} END {print max}' "$data_file")
    
    # Save statistics
    cat > "${data_file}.stats" << EOF
Average CPU: ${avg_cpu}%
Average Memory: ${avg_mem} MB
Average Threads: ${avg_threads}
Peak CPU: ${peak_cpu}%
Peak Memory: ${peak_mem} MB
EOF
    
    log_info "Statistics:"
    cat "${data_file}.stats"
}

compare_results() {
    local baseline=$1
    local new_version=$2
    
    log_info "Comparing results..."
    
    # Read baseline stats
    local baseline_cpu=$(grep "Average CPU:" "${baseline}.stats" | awk '{print $3}' | tr -d '%')
    local baseline_mem=$(grep "Average Memory:" "${baseline}.stats" | awk '{print $3}')
    
    # Read new version stats
    local new_cpu=$(grep "Average CPU:" "${new_version}.stats" | awk '{print $3}' | tr -d '%')
    local new_mem=$(grep "Average Memory:" "${new_version}.stats" | awk '{print $3}')
    
    # Calculate improvements
    local cpu_improvement=$(echo "scale=2; ($baseline_cpu - $new_cpu) / $baseline_cpu * 100" | bc)
    local mem_improvement=$(echo "scale=2; ($baseline_mem - $new_mem) / $baseline_mem * 100" | bc)
    
    # Generate comparison report
    local report="${RESULTS_DIR}/comparison_${TIMESTAMP}.txt"
    cat > "$report" << EOF
M5 Shared Decode Context - Performance Comparison
==================================================
Date: $(date)
Test Duration: ${DURATION} seconds
Sample Interval: ${SAMPLE_INTERVAL} seconds

Baseline (main branch):
-----------------------
Average CPU: ${baseline_cpu}%
Average Memory: ${baseline_mem} MB

New Version (m5-shared-decode):
--------------------------------
Average CPU: ${new_cpu}%
Average Memory: ${new_mem} MB

Improvements:
-------------
CPU: ${cpu_improvement}% reduction
Memory: ${mem_improvement}% reduction

Target Achievement:
-------------------
EOF
    
    # Check if targets met
    local cpu_target=60
    local mem_target=73
    
    if (( $(echo "$cpu_improvement >= $cpu_target" | bc -l) )); then
        echo "CPU: ✅ Target met (${cpu_target}% target, ${cpu_improvement}% achieved)" >> "$report"
    else
        echo "CPU: ❌ Target not met (${cpu_target}% target, ${cpu_improvement}% achieved)" >> "$report"
    fi
    
    if (( $(echo "$mem_improvement >= $mem_target" | bc -l) )); then
        echo "Memory: ✅ Target met (${mem_target}% target, ${mem_improvement}% achieved)" >> "$report"
    else
        echo "Memory: ❌ Target not met (${mem_target}% target, ${mem_improvement}% achieved)" >> "$report"
    fi
    
    # Display report
    echo ""
    cat "$report"
    echo ""
    
    log_success "Comparison report saved to: $report"
}

analyze_logs() {
    local log_file=$1
    local output=$2
    
    log_info "Analyzing decoder sharing from logs..."
    
    # Count decoder acquisitions
    local acquire_count=$(grep -c "Acquired shared decoder" "$log_file" || echo "0")
    local reuse_count=$(grep -c "Reusing existing decoder" "$log_file" || echo "0")
    
    # Get final ref count
    local final_refcount=$(grep "ref_count:" "$log_file" | tail -1 | grep -oP 'ref_count: \K\d+' || echo "0")
    
    cat > "$output" << EOF
Decoder Sharing Analysis:
-------------------------
New decoder creations: ${acquire_count}
Decoder reuses: ${reuse_count}
Final reference count: ${final_refcount}

Status: $( [[ $reuse_count -gt 0 ]] && echo "✅ Sharing working" || echo "❌ No sharing detected" )
EOF
    
    cat "$output"
}

# Main execution
main() {
    echo ""
    log_info "M5 Performance Testing Script"
    log_info "=============================="
    echo ""
    
    check_prerequisites
    
    # Create results directory
    mkdir -p "$RESULTS_DIR"
    
    # Save current branch
    local original_branch=$(git branch --show-current)
    
    # Test baseline (main)
    log_info "Phase 1: Baseline testing (main branch)"
    git checkout main
    local baseline_data="${RESULTS_DIR}/baseline_${TIMESTAMP}.csv"
    measure_performance "main" "$baseline_data" || {
        log_error "Baseline test failed!"
        git checkout "$original_branch"
        exit 1
    }
    
    # Test new version (m5-shared-decode)
    log_info "Phase 2: New version testing (m5-shared-decode branch)"
    git checkout m5-shared-decode
    local new_data="${RESULTS_DIR}/m5_${TIMESTAMP}.csv"
    measure_performance "m5-shared-decode" "$new_data" || {
        log_error "New version test failed!"
        git checkout "$original_branch"
        exit 1
    }
    
    # Analyze decoder sharing
    analyze_logs "${new_data}.log" "${new_data}.sharing"
    
    # Compare results
    compare_results "$baseline_data" "$new_data"
    
    # Restore original branch
    git checkout "$original_branch"
    
    log_success "Testing complete!"
    log_info "Results available in: $RESULTS_DIR"
}

# Run main function
main "$@"
