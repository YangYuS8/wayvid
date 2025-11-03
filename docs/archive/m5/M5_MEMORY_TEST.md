# Memory Optimization Testing Guide

本文档描述如何测试Issue #14的内存优化效果。

## 测试目标

- **基线内存**: ~380MB (v0.3.0多显示器场景)
- **优化目标**: ~100MB (减少73%)
- **验证点**: 内存稳定性、无泄漏、压力响应

## 快速测试 (5分钟)

### 1. 准备测试环境

确保有测试配置文件:

```bash
# 检查是否存在
ls test-config.yaml

# 如果不存在,创建一个
cat > test-config.yaml << 'EOF'
source:
  type: File
  path: "/usr/share/backgrounds/test.mp4"  # 替换为实际视频路径
layout: Fill
loop: true
hwdec: true
power:
  max_memory_mb: 100
  max_buffers: 8
EOF
```

### 2. 运行内存测试

```bash
# 测试60秒 (默认)
./scripts/test_memory_usage.sh

# 或指定时长
./scripts/test_memory_usage.sh 120  # 测试120秒

# 使用自定义配置
./scripts/test_memory_usage.sh 60 /path/to/config.yaml
```

### 3. 查看结果

测试完成后会显示:
- 平均/最小/最大内存使用
- 内存增长情况(检测泄漏)
- 解码器共享状态
- 内存压力事件

## 详细测试步骤

### Phase 1: 基准测试

1. **切换到main分支** (未优化版本):
```bash
git checkout main
cargo build --release
```

2. **运行基准测试**:
```bash
./scripts/test_memory_usage.sh 300  # 5分钟测试
```

3. **保存基准数据**:
```bash
# 假设输出在 test_results/memory_YYYYMMDD_HHMMSS.csv
cd test_results
grep "RSS (MB):" app_*.log | tail -1 > baseline_memory.txt
```

### Phase 2: 优化版本测试

1. **切换到优化分支**:
```bash
git checkout m5-memory-opt
cargo build --release
```

2. **运行优化测试**:
```bash
./scripts/test_memory_usage.sh 300
```

3. **自动对比**:
脚本会自动与baseline_memory.txt对比并显示改进百分比。

### Phase 3: 压力测试

测试长时间运行的稳定性:

```bash
# 30分钟压力测试
./scripts/test_memory_usage.sh 1800

# 检查结果
# 1. 内存增长应该 < 10%
# 2. 不应有Critical压力事件
# 3. RSS峰值应在100MB以下
```

## 测试场景

### 场景1: 单显示器

```yaml
# test-config-single.yaml
source:
  type: File
  path: "/path/to/video.mp4"
```

预期: ~30-40MB (单解码器)

### 场景2: 双显示器 (相同视频)

```yaml
# test-config-dual.yaml
source:
  type: File
  path: "/path/to/video.mp4"

# 系统会检测两个输出使用相同视频,共享解码器
```

预期: 
- ~40-50MB (共享解码器,略高于单显示器)
- 应看到 "Reusing existing decoder" 日志

### 场景3: 三显示器 (不同视频)

```yaml
# test-config-triple.yaml
source:
  type: File
  path: "/default.mp4"

per_output:
  "DP-1":
    source:
      type: File
      path: "/video1.mp4"
  "HDMI-1":
    source:
      type: File
      path: "/video2.mp4"
```

预期:
- ~90-100MB (3个独立解码器)
- 接近配置的max_memory_mb限制

## 分析工具

### 查看内存曲线

```bash
# 绘制RSS随时间变化 (需要gnuplot)
gnuplot <<EOF
set terminal png size 800,600
set output 'memory_usage.png'
set xlabel 'Time (seconds)'
set ylabel 'Memory (MB)'
set title 'Memory Usage Over Time'
plot 'test_results/memory_*.csv' using 1:(\$3/1024) with lines title 'RSS'
EOF
```

### 检查特定事件

```bash
cd test_results

# 查看所有内存统计日志
grep "Memory after" app_*.log

# 查看压力事件
grep "pressure" app_*.log

# 查看解码器创建/复用
grep -E "(Creating|Reusing)" app_*.log

# 查看缓冲池操作
grep -E "(pool|buffer)" app_*.log
```

## 成功标准

### 必须满足 ✅
1. **内存减少 > 50%**: 从~380MB降至<190MB
2. **无内存泄漏**: 增长 < 10% in 30分钟
3. **稳定运行**: 无崩溃,无OOM

### 理想目标 🎯
1. **内存减少 > 73%**: 降至~100MB
2. **解码器共享**: 相同视频只有1个解码器
3. **压力响应**: High压力自动清理,无Critical

### 验收测试 📋

运行完整测试套件:

```bash
# 1. 单元测试
cargo test --all-features

# 2. 短期内存测试 (60秒)
./scripts/test_memory_usage.sh 60

# 3. 长期稳定性 (30分钟)
./scripts/test_memory_usage.sh 1800

# 4. 检查所有结果
ls -lh test_results/
```

## 故障排除

### 问题: 内存仍然很高

**检查**:
```bash
# 1. 确认优化代码已编译
cargo build --release --all-features
./target/release/wayvid --version

# 2. 检查配置是否生效
grep "max_memory_mb" test-config.yaml

# 3. 查看是否有压力事件
grep "pressure" test_results/app_*.log
```

**解决**:
- 降低max_memory_mb限制
- 减少max_buffers数量
- 检查视频分辨率(4K消耗更多内存)

### 问题: 频繁Critical压力

**原因**: 内存限制过严

**解决**:
```yaml
power:
  max_memory_mb: 150  # 提高限制
  max_buffers: 12     # 增加缓冲区
```

### 问题: 性能下降

**检查**:
```bash
# 查看帧率和卡顿
grep "render" test_results/app_*.log | tail -100
```

**平衡**: 内存和性能需要权衡,找到最佳配置。

## 结果报告模板

```markdown
## Memory Test Results

**Date**: 2025-10-24
**Branch**: m5-memory-opt
**Commit**: [commit hash]

### Configuration
- Duration: 300s
- Displays: 2
- Video: 1920x1080 @ 60fps

### Results
| Metric | Baseline | Optimized | Change |
|--------|----------|-----------|--------|
| Avg RSS | 380 MB | 95 MB | -75% ✅ |
| Peak RSS | 420 MB | 110 MB | -74% ✅ |
| Memory Growth | +15% ⚠️ | +3% ✅ | Better |

### Observations
- ✅ Decoder sharing working (1 created, 1 reused)
- ✅ No critical pressure events
- ✅ Memory stable over time
- ⚠️ Minor high pressure at startup

### Conclusion
Memory optimization successful! Target achieved.
```

## 下一步

测试通过后:
1. 提交测试结果
2. 更新文档
3. 创建PR
4. 关闭Issue #14

---

**Questions?** 查看日志或运行: `./scripts/test_memory_usage.sh --help`
