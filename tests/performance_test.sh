#!/bin/bash
# M2 Phase 7: 性能测试脚本

echo "=========================================="
echo "M2 Phase 7: 性能测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 启动程序并在后台运行
echo "启动 wayvid 进行性能测试..."
./target/release/wayvid run > /tmp/wayvid_perf.log 2>&1 &
WAYVID_PID=$!

echo "PID: $WAYVID_PID"
sleep 3  # 等待完全启动

# 检查进程是否还在运行
if ! ps -p $WAYVID_PID > /dev/null 2>&1; then
    echo -e "${RED}✗${NC} 程序启动失败或已崩溃"
    cat /tmp/wayvid_perf.log
    exit 1
fi

echo -e "${GREEN}✓${NC} 程序已启动\n"

# 性能采样
echo "采集性能数据 (10 秒)..."
echo ""

# CPU 使用率
echo "=== CPU 使用率 ==="
CPU_SAMPLES=()
for i in {1..10}; do
    CPU=$(ps -p $WAYVID_PID -o %cpu= 2>/dev/null | tr -d ' ')
    if [ -n "$CPU" ]; then
        CPU_SAMPLES+=($CPU)
        echo "样本 $i: ${CPU}%"
    fi
    sleep 1
done

# 计算平均 CPU
if [ ${#CPU_SAMPLES[@]} -gt 0 ]; then
    AVG_CPU=$(echo "${CPU_SAMPLES[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {print sum/NR}')
    echo -e "\n${YELLOW}平均 CPU 使用率: ${AVG_CPU}%${NC}\n"
fi

# 内存使用
echo "=== 内存使用 ==="
MEM_KB=$(ps -p $WAYVID_PID -o rss= 2>/dev/null | tr -d ' ')
MEM_MB=$(echo "scale=2; $MEM_KB / 1024" | bc)
echo -e "RSS: ${MEM_MB} MB\n"

# VSZ (虚拟内存)
VSZ_KB=$(ps -p $WAYVID_PID -o vsz= 2>/dev/null | tr -d ' ')
VSZ_MB=$(echo "scale=2; $VSZ_KB / 1024" | bc)
echo -e "VSZ: ${VSZ_MB} MB\n"

# 线程数
THREADS=$(ps -p $WAYVID_PID -o nlwp= 2>/dev/null | tr -d ' ')
echo -e "线程数: ${THREADS}\n"

# 打开文件描述符
FD_COUNT=$(ls -1 /proc/$WAYVID_PID/fd 2>/dev/null | wc -l)
echo -e "文件描述符: ${FD_COUNT}\n"

# GPU 使用 (如果可用)
echo "=== GPU 信息 ==="
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader 2>/dev/null | grep $WAYVID_PID || echo "未检测到 NVIDIA GPU 使用"
elif command -v radeontop &> /dev/null; then
    echo "AMD GPU 检测需要 radeontop -d -"
else
    echo "未找到 GPU 监控工具"
fi
echo ""

# 帧率信息 (从日志)
echo "=== 日志摘要 ==="
tail -20 /tmp/wayvid_perf.log | grep -E "(render|frame|FPS|error|warn)" || echo "无相关日志"
echo ""

# 清理
echo "停止程序..."
kill $WAYVID_PID 2>/dev/null
wait $WAYVID_PID 2>/dev/null

echo ""
echo "=========================================="
echo "性能测试完成"
echo "=========================================="
echo ""
echo "性能总结:"
echo "  - 平均 CPU: ${AVG_CPU}%"
echo "  - 内存 (RSS): ${MEM_MB} MB"
echo "  - 线程数: ${THREADS}"
echo ""

# 性能评估
CPU_THRESHOLD=50
MEM_THRESHOLD=200

if (( $(echo "$AVG_CPU < $CPU_THRESHOLD" | bc -l) )); then
    echo -e "${GREEN}✓${NC} CPU 使用率良好 (< ${CPU_THRESHOLD}%)"
else
    echo -e "${YELLOW}⚠${NC} CPU 使用率较高 (> ${CPU_THRESHOLD}%)"
fi

if (( $(echo "$MEM_MB < $MEM_THRESHOLD" | bc -l) )); then
    echo -e "${GREEN}✓${NC} 内存使用良好 (< ${MEM_THRESHOLD} MB)"
else
    echo -e "${YELLOW}⚠${NC} 内存使用较高 (> ${MEM_THRESHOLD} MB)"
fi

echo ""
echo "完整日志保存在: /tmp/wayvid_perf.log"
