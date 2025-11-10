#!/usr/bin/env bash
#
# Quick test log analyzer
# Usage: ./scripts/analyze_test_log.sh <log_file>
#

LOG_FILE=${1:-"test.log"}

if [[ ! -f "$LOG_FILE" ]]; then
    echo "❌ Log file not found: $LOG_FILE"
    echo "Usage: $0 <log_file>"
    exit 1
fi

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  📊 M5 Test Log Analysis                                          ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Count decoder operations
acquire_count=$(grep -c "Acquired shared decoder" "$LOG_FILE" || echo "0")
reuse_count=$(grep -c "Reusing existing decoder" "$LOG_FILE" || echo "0")
release_count=$(grep -c "Released decoder" "$LOG_FILE" || echo "0")

echo "🔍 Decoder Sharing Statistics:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  New decoder acquisitions: $acquire_count"
echo "  Decoder reuses:           $reuse_count"
echo "  Decoder releases:         $release_count"
echo ""

if [[ $reuse_count -gt 0 ]]; then
    echo "  ✅ Decoder sharing is working!"
else
    echo "  ❌ No decoder sharing detected!"
fi
echo ""

# Get reference counts
echo "📈 Reference Count Timeline:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
grep "ref_count:" "$LOG_FILE" | tail -10 || echo "  No ref_count data found"
echo ""

# Check for errors
error_count=$(grep -i "error\|ERROR" "$LOG_FILE" | grep -v "error_code" | wc -l)
warn_count=$(grep -i "warn\|WARNING" "$LOG_FILE" | wc -l)

echo "⚠️  Issues Detected:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Errors:   $error_count"
echo "  Warnings: $warn_count"

if [[ $error_count -gt 0 ]]; then
    echo ""
    echo "  Recent errors:"
    grep -i "error" "$LOG_FILE" | grep -v "error_code" | tail -5 | sed 's/^/    /'
fi
echo ""

# Summary
echo "📋 Test Summary:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ $reuse_count -gt 0 ]] && [[ $error_count -eq 0 ]]; then
    echo "  ✅ TEST PASSED - Decoder sharing works correctly!"
elif [[ $reuse_count -gt 0 ]] && [[ $error_count -gt 0 ]]; then
    echo "  ⚠️  TEST PARTIAL - Sharing works but errors detected"
else
    echo "  ❌ TEST FAILED - Decoder sharing not working"
fi
echo ""
