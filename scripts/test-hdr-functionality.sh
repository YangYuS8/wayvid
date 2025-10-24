#!/bin/bash
# Automated HDR Functionality Tests
# ==================================
#
# This script performs automated tests of HDR functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to print colored output
print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_test() {
    echo -e "${CYAN}▶ Test $1: $2${NC}"
}

print_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

print_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

print_skip() {
    echo -e "${YELLOW}⊘ SKIP${NC}: $1"
}

# Function to create temp config
create_test_config() {
    local config_file="$1"
    shift
    cat > "$config_file" "$@"
}

# Function to check log for pattern
check_log() {
    local log_file="$1"
    local pattern="$2"
    
    if grep -q "$pattern" "$log_file" 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

# Start tests
print_header "HDR Automated Functionality Tests"
echo ""
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo "Platform: $(uname -s) $(uname -m)"
echo ""

# Build wayvid
print_header "Building wayvid"
cargo build --release 2>&1 | tail -3
if [ $? -eq 0 ]; then
    print_pass "Build successful"
else
    print_fail "Build failed"
    exit 1
fi
echo ""

# Kill any running instances
pkill -9 wayvid 2>/dev/null || true
sleep 1

# Test suite
TEST_LOG="/tmp/wayvid-hdr-test.log"
CONFIG_FILE="/tmp/wayvid-hdr-test-config.yaml"

print_header "Test Suite: HDR Configuration Tests"
echo ""

# Test 1: Config Validation - Invalid Parameter
print_test "1.1" "Configuration Validation - Invalid Parameter"
TESTS_RUN=$((TESTS_RUN + 1))

cat > "$CONFIG_FILE" << 'EOF'
source:
  type: file
  path: "/dev/null"

hdr_mode: auto

tone_mapping:
  algorithm: hable
  param: 15.0
  compute_peak: true
  mode: hybrid
EOF

timeout 2s ./target/release/wayvid run --config "$CONFIG_FILE" --log-level debug > "$TEST_LOG" 2>&1 || true

if check_log "$TEST_LOG" "clamping"; then
    print_pass "Invalid parameter detected and clamped"
else
    print_fail "Parameter validation not working"
fi
echo ""

# Test 2: HDR Mode - Auto
print_test "1.2" "HDR Mode Auto Configuration"
TESTS_RUN=$((TESTS_RUN + 1))

cat > "$CONFIG_FILE" << 'EOF'
source:
  type: file
  path: "/dev/null"

hdr_mode: auto

tone_mapping:
  algorithm: hable
  param: 1.0
  compute_peak: true
  mode: hybrid
EOF

timeout 2s ./target/release/wayvid run --config "$CONFIG_FILE" --log-level debug > "$TEST_LOG" 2>&1 || true

if check_log "$TEST_LOG" "Configuring HDR"; then
    print_pass "HDR configuration activated"
else
    print_fail "HDR configuration not activated"
fi
echo ""

# Test 3: Algorithm Configuration
print_test "1.3" "Tone Mapping Algorithm Selection"
TESTS_RUN=$((TESTS_RUN + 1))

for algo in hable mobius reinhard bt2390; do
    cat > "$CONFIG_FILE" << EOF
source:
  type: file
  path: "/dev/null"

hdr_mode: auto

tone_mapping:
  algorithm: $algo
  param: 1.0
  compute_peak: true
  mode: hybrid
EOF

    timeout 2s ./target/release/wayvid run --config "$CONFIG_FILE" --log-level debug > "$TEST_LOG" 2>&1 || true
    
    if check_log "$TEST_LOG" "Algorithm: $algo"; then
        print_info "  ✓ $algo algorithm configured"
    else
        print_fail "Algorithm $algo not configured correctly"
        break
    fi
done

if [ $? -eq 0 ]; then
    print_pass "All algorithms configured correctly"
else
    print_fail "Some algorithms failed configuration"
fi
echo ""

# Test 4: Mode Configuration
print_test "1.4" "Tone Mapping Mode Selection"
TESTS_RUN=$((TESTS_RUN + 1))

success=true
for mode in hybrid rgb luma auto; do
    cat > "$CONFIG_FILE" << EOF
source:
  type: file
  path: "/dev/null"

hdr_mode: auto

tone_mapping:
  algorithm: hable
  param: 1.0
  compute_peak: true
  mode: $mode
EOF

    timeout 2s ./target/release/wayvid run --config "$CONFIG_FILE" --log-level debug > "$TEST_LOG" 2>&1 || true
    
    if check_log "$TEST_LOG" "Mode: $mode"; then
        print_info "  ✓ $mode mode configured"
    else
        print_info "  ✗ $mode mode failed"
        success=false
    fi
done

if $success; then
    print_pass "All modes configured correctly"
else
    print_fail "Some modes failed configuration"
fi
echo ""

# Test 5: HDR Mode - Force
print_test "1.5" "HDR Mode Force"
TESTS_RUN=$((TESTS_RUN + 1))

cat > "$CONFIG_FILE" << 'EOF'
source:
  type: file
  path: "/dev/null"

hdr_mode: force

tone_mapping:
  algorithm: hable
  param: 1.0
  compute_peak: true
  mode: hybrid
EOF

timeout 2s ./target/release/wayvid run --config "$CONFIG_FILE" --log-level debug > "$TEST_LOG" 2>&1 || true

if check_log "$TEST_LOG" "tone mapping"; then
    print_pass "Force mode activates tone mapping"
else
    print_fail "Force mode not working"
fi
echo ""

# Test 6: HDR Mode - Disable
print_test "1.6" "HDR Mode Disable"
TESTS_RUN=$((TESTS_RUN + 1))

cat > "$CONFIG_FILE" << 'EOF'
source:
  type: file
  path: "/dev/null"

hdr_mode: disable

tone_mapping:
  algorithm: hable
  param: 1.0
  compute_peak: true
  mode: hybrid
EOF

timeout 2s ./target/release/wayvid run --config "$CONFIG_FILE" --log-level debug > "$TEST_LOG" 2>&1 || true

if ! check_log "$TEST_LOG" "tone mapping configured"; then
    print_pass "Disable mode skips HDR processing"
else
    print_fail "Disable mode not working"
fi
echo ""

# Test 7: Dynamic Peak Detection
print_test "1.7" "Dynamic Peak Detection Configuration"
TESTS_RUN=$((TESTS_RUN + 1))

cat > "$CONFIG_FILE" << 'EOF'
source:
  type: file
  path: "/dev/null"

hdr_mode: auto

tone_mapping:
  algorithm: hable
  param: 1.0
  compute_peak: true
  mode: hybrid
EOF

timeout 2s ./target/release/wayvid run --config "$CONFIG_FILE" --log-level debug > "$TEST_LOG" 2>&1 || true

if check_log "$TEST_LOG" "Dynamic peak detection: enabled"; then
    print_pass "Dynamic peak detection configured"
else
    print_fail "Dynamic peak detection configuration failed"
fi
echo ""

# Test 8: Invalid Mode Validation
print_test "1.8" "Invalid Mode Validation"
TESTS_RUN=$((TESTS_RUN + 1))

cat > "$CONFIG_FILE" << 'EOF'
source:
  type: file
  path: "/dev/null"

hdr_mode: auto

tone_mapping:
  algorithm: hable
  param: 1.0
  compute_peak: true
  mode: invalid_mode_xyz
EOF

timeout 2s ./target/release/wayvid run --config "$CONFIG_FILE" --log-level debug > "$TEST_LOG" 2>&1 || true

if check_log "$TEST_LOG" "Invalid tone mapping mode"; then
    print_pass "Invalid mode detected and warned"
else
    print_fail "Invalid mode validation not working"
fi
echo ""

# Cleanup
rm -f "$TEST_LOG" "$CONFIG_FILE"

# Summary
print_header "Test Summary"
echo ""
echo "Total Tests: $TESTS_RUN"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}  ✓ ALL TESTS PASSED${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}  ✗ SOME TESTS FAILED${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 1
fi
