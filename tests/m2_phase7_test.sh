#!/bin/bash
# M2 Phase 7: 综合测试脚本

set -e

echo "=========================================="
echo "M2 Phase 7: 综合测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试结果跟踪
TESTS_PASSED=0
TESTS_FAILED=0

# 测试函数
test_pass() {
    echo -e "${GREEN}✓${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

test_fail() {
    echo -e "${RED}✗${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

test_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# 1. 编译测试
echo "=== 测试 1: 编译测试 ==="
if cargo build --release --features video-mpv 2>&1 | grep -q "Finished"; then
    test_pass "编译成功"
else
    test_fail "编译失败"
    exit 1
fi
echo ""

# 2. 基础启动测试
echo "=== 测试 2: 基础启动测试 ==="
if timeout 2 ./target/release/wayvid run 2>&1 | grep -q "wayvid version"; then
    test_pass "程序可以启动"
else
    test_fail "程序启动失败"
fi
echo ""

# 3. 输出检测测试
echo "=== 测试 3: 输出检测 ==="
OUTPUT_COUNT=$(timeout 2 ./target/release/wayvid run 2>&1 | grep -o "Outputs discovered: [0-9]*" | grep -o "[0-9]*" || echo "0")
if [ "$OUTPUT_COUNT" -gt 0 ]; then
    test_pass "检测到 $OUTPUT_COUNT 个输出"
else
    test_fail "未检测到输出"
fi
echo ""

# 4. EGL 初始化测试
echo "=== 测试 4: EGL 初始化 ==="
if timeout 2 ./target/release/wayvid run 2>&1 | grep -q "EGL context created successfully"; then
    test_pass "EGL 上下文初始化成功"
else
    test_fail "EGL 初始化失败"
fi
echo ""

# 5. MPV 初始化测试
echo "=== 测试 5: MPV 播放器初始化 ==="
if timeout 2 ./target/release/wayvid run 2>&1 | grep -q "MPV initialized successfully"; then
    test_pass "MPV 播放器初始化成功"
else
    test_fail "MPV 初始化失败"
fi
echo ""

# 6. 渲染上下文测试
echo "=== 测试 6: 渲染上下文 ==="
if timeout 2 ./target/release/wayvid run 2>&1 | grep -q "Render context created successfully"; then
    test_pass "渲染上下文创建成功"
else
    test_fail "渲染上下文创建失败"
fi
echo ""

# 7. xdg-output 协议测试
echo "=== 测试 7: xdg-output 协议支持 ==="
if timeout 2 ./target/release/wayvid run 2>&1 | grep -q "Bound zxdg_output_manager_v1"; then
    test_pass "xdg-output 协议绑定成功"
else
    test_info "xdg-output 协议未绑定 (合成器可能不支持)"
fi
echo ""

# 8. 电源管理测试
echo "=== 测试 8: 电源管理 ==="
test_info "电池状态检测需要手动验证"
if [ -d "/sys/class/power_supply" ]; then
    BATTERY_COUNT=$(ls /sys/class/power_supply/ | grep -c "BAT" || echo "0")
    if [ "$BATTERY_COUNT" -gt 0 ]; then
        test_pass "检测到 $BATTERY_COUNT 个电池设备"
        for bat in /sys/class/power_supply/BAT*; do
            if [ -f "$bat/status" ]; then
                STATUS=$(cat "$bat/status")
                test_info "$(basename $bat) 状态: $STATUS"
            fi
        done
    else
        test_info "未检测到电池设备 (台式机?)"
    fi
else
    test_info "/sys/class/power_supply 不存在"
fi
echo ""

# 9. 配置文件测试
echo "=== 测试 9: 配置文件解析 ==="
if [ -f "$HOME/.config/wayvid/config.yaml" ]; then
    test_pass "配置文件存在"
    test_info "路径: $HOME/.config/wayvid/config.yaml"
else
    test_info "配置文件不存在 (使用默认配置)"
fi
echo ""

# 10. 二进制大小
echo "=== 测试 10: 二进制大小 ==="
BINARY_SIZE=$(du -h target/release/wayvid | cut -f1)
test_info "二进制大小: $BINARY_SIZE"
echo ""

# 总结
echo "=========================================="
echo "测试总结"
echo "=========================================="
echo -e "${GREEN}通过: $TESTS_PASSED${NC}"
echo -e "${RED}失败: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有关键测试通过!${NC}"
    exit 0
else
    echo -e "${RED}✗ 部分测试失败${NC}"
    exit 1
fi
