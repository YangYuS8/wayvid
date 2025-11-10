#!/usr/bin/env bash
# Quick development check - faster feedback during development
# Only runs essential checks

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Quick Dev Check...${NC}"

# 1. Format and fix
echo -e "${BLUE}→${NC} Formatting code..."
cargo fmt --all

# 2. Quick clippy on current code (not all targets)
echo -e "${BLUE}→${NC} Running quick clippy..."
cargo clippy --all-features -- -D warnings

# 3. Quick compile check
echo -e "${BLUE}→${NC} Checking compilation..."
cargo check --all-features

echo -e "\n${GREEN}✓ Quick checks passed!${NC}\n"
echo "Tip: Run './scripts/pre-push-check.sh' before pushing"
