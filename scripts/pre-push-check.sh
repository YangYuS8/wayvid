#!/usr/bin/env bash
# Pre-push quality check script
# Mimics CI checks but runs locally for faster feedback

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print section headers
print_section() {
    echo -e "\n${BLUE}==>${NC} ${1}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓${NC} ${1}"
}

# Function to print error
print_error() {
    echo -e "${RED}✗${NC} ${1}"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠${NC} ${1}"
}

# Start timer
START_TIME=$(date +%s)

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Wayvid Pre-Push Quality Checks (v0.5) ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"

# 1. Format check
print_section "Checking code formatting..."
if cargo fmt --all -- --check > /dev/null 2>&1; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting failed"
    echo "Run: cargo fmt --all"
    exit 1
fi

# 2. Clippy (workspace mode)
print_section "Running Clippy (workspace)..."
if RUSTFLAGS="-D warnings" cargo clippy --workspace --all-targets 2>&1 | grep -q "Finished"; then
    print_success "Clippy checks passed"
else
    print_error "Clippy found issues"
    echo "Run: cargo clippy --workspace --all-targets -- -D warnings"
    exit 1
fi

# 3. Cargo check (workspace)
print_section "Checking compilation (workspace)..."
if cargo check --workspace --quiet 2>&1 | tail -1 | grep -q "Finished"; then
    print_success "Workspace compiles"
else
    print_error "Workspace failed to compile"
    exit 1
fi

# 4. Tests (workspace)
if [ "${SKIP_TESTS:-0}" = "0" ]; then
    print_section "Running tests (workspace)..."
    if cargo test --workspace --quiet 2>&1 | tail -1 | grep -q "test result"; then
        print_success "Tests passed"
    else
        print_error "Tests failed"
        echo "Run: cargo test --workspace"
        exit 1
    fi
else
    print_warning "Tests skipped (SKIP_TESTS=1)"
fi

# Calculate elapsed time
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo -e "\n${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  All checks passed! ✓                  ║${NC}"
echo -e "${GREEN}║  Time: ${ELAPSED}s                               ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo -e "\n${BLUE}Ready to push!${NC}\n"
