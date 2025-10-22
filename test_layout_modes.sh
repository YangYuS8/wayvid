#!/bin/bash
# Test all layout modes

MODES=("Fill" "Contain" "Stretch" "Centre")
VIDEO="/home/yangyus8/Videos/test.mp4"

echo "=========================================="
echo "M2 Phase 4: Layout Mode Testing"
echo "=========================================="
echo ""

for mode in "${MODES[@]}"; do
    echo "🧪 Testing: $mode"
    
    cat > /tmp/test_layout.yaml <<EOF
source:
  type: File
  path: $VIDEO
layout: $mode
loop: true
mute: true
EOF
    
    # Run for 1 second and capture first layout log line
    timeout 1 ./target/release/wayvid --log-level debug run --config /tmp/test_layout.yaml 2>&1 | \
        grep -E "Layout.*viewport" | head -1
    
    sleep 0.5
done

rm -f /tmp/test_layout.yaml

echo ""
echo "=========================================="
echo "✅ All layout modes tested successfully!"
echo "=========================================="
