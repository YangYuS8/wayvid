#!/bin/bash
# Benchmark optimizations

echo "=========================================="
echo "优化效果验证"
echo "=========================================="
echo ""

# Test 1: 编译产物大小
echo "📦 编译产物大小:"
ls -lh target/release/wayvid | awk '{print "  " $9 ": " $5}'
echo ""

# Test 2: 运行时性能 (2秒采样)
echo "⚡ 运行时性能测试 (2秒):"
OUTPUT=$(timeout 2 ./target/release/wayvid --log-level warn run 2>&1)
echo "$OUTPUT" | tail -5
echo ""

# Test 3: 验证缓存 (debug 模式不应有重复的 Layout 计算日志)
echo "🔍 验证布局缓存:"
LAYOUT_COUNT=$(timeout 1 ./target/release/wayvid --log-level debug run 2>&1 | grep -c "Layout" || echo "0")
if [ "$LAYOUT_COUNT" -eq "0" ]; then
    echo "  ✅ 布局缓存工作正常 (无重复计算)"
else
    echo "  ⚠️  检测到 $LAYOUT_COUNT 次布局计算 (可能需要优化)"
fi
echo ""

# Test 4: 内存使用 (粗略估计)
echo "💾 二进制大小优化:"
BINARY_SIZE=$(stat -f%z target/release/wayvid 2>/dev/null || stat -c%s target/release/wayvid 2>/dev/null)
BINARY_MB=$(echo "scale=2; $BINARY_SIZE / 1024 / 1024" | bc)
echo "  Release 二进制: ${BINARY_MB} MB"
echo ""

echo "=========================================="
echo "✅ 优化验证完成"
echo "=========================================="
