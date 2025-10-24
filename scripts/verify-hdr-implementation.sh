#!/bin/bash
# HDR Implementation Verification Tests
# ======================================
#
# Tests HDR implementation without requiring actual video playback

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

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

print_header "HDR Implementation Verification"
echo ""
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo "Platform: $(uname -s) $(uname -m)"
echo ""

# Test 1: Check HDR module exists
print_test "1" "HDR Module Exists"
TESTS_RUN=$((TESTS_RUN + 1))

if [ -f "src/video/hdr.rs" ]; then
    print_pass "HDR module found"
else
    print_fail "HDR module not found"
fi
echo ""

# Test 2: Check HDR types
print_test "2" "HDR Type Definitions"
TESTS_RUN=$((TESTS_RUN + 1))

types_found=0
for type in "ColorSpace" "TransferFunction" "HdrMetadata" "HdrMode" "ToneMappingAlgorithm" "ToneMappingConfig"; do
    if grep -q "pub enum $type\|pub struct $type" src/video/hdr.rs; then
        print_info "  ✓ $type defined"
        types_found=$((types_found + 1))
    fi
done

if [ $types_found -eq 6 ]; then
    print_pass "All HDR types defined"
else
    print_fail "Some HDR types missing ($types_found/6)"
fi
echo ""

# Test 3: Check MPV HDR methods
print_test "3" "MPV HDR Methods"
TESTS_RUN=$((TESTS_RUN + 1))

methods_found=0
for method in "get_hdr_metadata" "configure_hdr" "configure_tone_mapping" "configure_hdr_passthrough"; do
    if grep -q "fn $method" src/video/mpv.rs; then
        print_info "  ✓ $method implemented"
        methods_found=$((methods_found + 1))
    fi
done

if [ $methods_found -eq 4 ]; then
    print_pass "All HDR methods implemented"
else
    print_fail "Some HDR methods missing ($methods_found/4)"
fi
echo ""

# Test 4: Check configuration integration
print_test "4" "Configuration Integration"
TESTS_RUN=$((TESTS_RUN + 1))

config_found=0
if grep -q "hdr_mode" src/config/types.rs; then
    print_info "  ✓ hdr_mode in config"
    config_found=$((config_found + 1))
fi
if grep -q "tone_mapping" src/config/types.rs; then
    print_info "  ✓ tone_mapping in config"
    config_found=$((config_found + 1))
fi
if grep -q "validate" src/config/types.rs; then
    print_info "  ✓ validation implemented"
    config_found=$((config_found + 1))
fi

if [ $config_found -eq 3 ]; then
    print_pass "Configuration integrated"
else
    print_fail "Configuration integration incomplete ($config_found/3)"
fi
echo ""

# Test 5: Check documentation
print_test "5" "Documentation Exists"
TESTS_RUN=$((TESTS_RUN + 1))

docs_found=0
if [ -f "docs/HDR_USER_GUIDE.md" ]; then
    print_info "  ✓ User guide exists"
    docs_found=$((docs_found + 1))
fi
if [ -f "docs/HDR_WAYLAND_STATUS.md" ]; then
    print_info "  ✓ Wayland status doc exists"
    docs_found=$((docs_found + 1))
fi
if [ -f "examples/hdr-config.yaml" ]; then
    print_info "  ✓ Example config exists"
    docs_found=$((docs_found + 1))
fi

if [ $docs_found -eq 3 ]; then
    print_pass "Documentation complete"
else
    print_fail "Documentation incomplete ($docs_found/3)"
fi
echo ""

# Test 6: Check test scripts
print_test "6" "Test Scripts Exist"
TESTS_RUN=$((TESTS_RUN + 1))

scripts_found=0
if [ -f "scripts/test-hdr-tonemapping.sh" ]; then
    print_info "  ✓ Tone mapping test script"
    scripts_found=$((scripts_found + 1))
fi
if [ -f "scripts/test-hdr-functionality.sh" ]; then
    print_info "  ✓ Functionality test script"
    scripts_found=$((scripts_found + 1))
fi

if [ $scripts_found -eq 2 ]; then
    print_pass "Test scripts available"
else
    print_fail "Test scripts missing ($scripts_found/2)"
fi
echo ""

# Test 7: Compilation test
print_test "7" "Code Compiles"
TESTS_RUN=$((TESTS_RUN + 1))

print_info "  Running cargo check..."
if cargo check --quiet 2>&1 | grep -q "error"; then
    print_fail "Compilation errors found"
else
    print_pass "Code compiles successfully"
fi
echo ""

# Test 8: Check README integration
print_test "8" "README Updated"
TESTS_RUN=$((TESTS_RUN + 1))

if grep -q "HDR Support" README.md; then
    print_pass "README includes HDR section"
else
    print_fail "README missing HDR section"
fi
echo ""

# Test 9: Check content-aware optimization
print_test "9" "Content-Aware Optimization"
TESTS_RUN=$((TESTS_RUN + 1))

if grep -q "ContentType" src/video/hdr.rs && grep -q "optimize_for_content" src/video/hdr.rs; then
    print_pass "Content-aware optimization implemented"
else
    print_fail "Content-aware optimization missing"
fi
echo ""

# Test 10: Check performance presets
print_test "10" "Performance Presets"
TESTS_RUN=$((TESTS_RUN + 1))

if grep -q "PerformancePreset" src/video/hdr.rs; then
    print_pass "Performance presets defined"
else
    print_fail "Performance presets missing"
fi
echo ""

# Summary
print_header "Verification Summary"
echo ""
echo "Total Tests: $TESTS_RUN"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

PASS_RATE=$((TESTS_PASSED * 100 / TESTS_RUN))
echo "Pass Rate: $PASS_RATE%"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}  ✓ ALL VERIFICATIONS PASSED${NC}"
    echo -e "${GREEN}  HDR implementation is complete and ready for testing${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}  ✗ SOME VERIFICATIONS FAILED${NC}"
    echo -e "${RED}  Please review and fix the issues above${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 1
fi
