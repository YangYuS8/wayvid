#!/bin/bash
# 测试 wayvid 与 swww 冲突的脚本

set -e

echo "=== wayvid 与 swww 冲突测试 ==="
echo

# 检查 swww 状态
echo "1. 检查 swww-daemon 状态..."
if pgrep -x swww-daemon > /dev/null; then
    echo "   ✓ swww-daemon 正在运行"
    SWWW_RUNNING=true
else
    echo "   ✗ swww-daemon 未运行"
    SWWW_RUNNING=false
fi
echo

# 启动 wayvid（后台，2秒后自动退出）
echo "2. 启动 wayvid（2秒测试）..."
timeout 2 ./target/release/wayvid run --config /tmp/test-fixed.yaml &> /tmp/wayvid-test.log || true

# 检查警告信息
echo "3. 检查冲突警告..."
if grep -q "Detected swww-daemon" /tmp/wayvid-test.log; then
    echo "   ✓ 成功检测到 swww-daemon 冲突"
    echo "   ✓ 显示了警告信息"
else
    if [ "$SWWW_RUNNING" = true ]; then
        echo "   ✗ 未检测到 swww-daemon（但它在运行）"
        exit 1
    else
        echo "   ✓ 未检测到冲突（swww 未运行）"
    fi
fi
echo

# 显示警告内容
if grep -q "⚠️" /tmp/wayvid-test.log; then
    echo "4. 警告信息内容："
    grep "⚠️" /tmp/wayvid-test.log | sed 's/^/   /'
    echo
fi

# 清理
rm -f /tmp/wayvid-test.log

echo "=== 测试完成 ==="
echo
echo "建议："
if [ "$SWWW_RUNNING" = true ]; then
    echo "- 运行 'killall swww-daemon' 停止 swww"
    echo "- 从 compositor 配置中移除 swww 自动启动"
    echo "- 重启 compositor 或手动启动 wayvid"
fi
