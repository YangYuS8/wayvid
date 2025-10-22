#!/bin/bash
# Test all layout modes

MODES=("Fill" "Contain" "Stretch" "Centre")
VIDEO="/home/yangyus8/Videos/test.mp4"

for mode in "${MODES[@]}"; do
    echo "========================================"
    echo "Testing Layout Mode: $mode"
    echo "========================================"
    
    # Create temporary config
    cat > /tmp/wayvid_test.toml <<EOF
layout = "$mode"
loop = true
mute = true

[source]
type = "file"
path = "$VIDEO"
EOF
    
    # Run for 3 seconds and save full log
    timeout 3 ./target/release/wayvid --log-level debug run --config /tmp/wayvid_test.toml > /tmp/wayvid_${mode}.log 2>&1
    
    # Extract layout info
    grep -E "Layout.*video.*viewport" /tmp/wayvid_${mode}.log | head -1
    
    echo ""
    sleep 1
done

rm -f /tmp/wayvid_test.toml
echo ""
echo "✅ All layout modes tested!"
echo ""
echo "详细日志已保存到:"
ls -lh /tmp/wayvid_*.log
