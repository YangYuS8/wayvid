# M5 Quick Testing Checklist

## ⚡ 快速验证步骤 (10分钟)

如果您没有时间运行完整测试,可以按照此清单快速验证核心功能:

### 1. 准备测试环境 (2分钟)

```bash
# 确保在正确的分支
git checkout m5-shared-decode

# 快速编译
cargo build --release --features video-mpv

# 创建测试配置(如果没有)
mkdir -p ~/.config/wayvid
cat > ~/.config/wayvid/test-config.toml << 'EOF'
[video]
source = { file = "/home/yangyus8/test.mp4" }
loop_playback = true

[video.hwdec]
mode = "auto"

[render]
layout = "contain"
EOF
```

### 2. 验证解码器共享 (3分钟)

```bash
# 启动 wayvid 并观察日志
RUST_LOG=wayvid::video::shared_decode=debug \
  ./target/release/wayvid 2>&1 | tee test.log
```

**✅ 成功标志**:
- 第一个显示器: `Acquired shared decoder for source: file=...`
- 第二个显示器: `♻️ Reusing existing decoder (ref_count: 2)`
- 第三个显示器: `♻️ Reusing existing decoder (ref_count: 3)`

**❌ 失败标志**:
- 多次看到 `Acquired shared decoder` 对相同的源
- 没有看到 `Reusing existing decoder`
- 每个显示器都创建了新的解码器

### 3. 快速性能检查 (3分钟)

```bash
# 在另一个终端运行 htop
htop

# 观察:
# - 只应该有 1 个 wayvid 进程(不是3个)
# - CPU 使用率应该比旧版本低很多
# - 内存占用应该在 100-150MB 左右(不是 300-400MB)
```

### 4. 稳定性测试 (2分钟)

```bash
# 让 wayvid 运行 2 分钟
# 观察:
# - 画面是否正常
# - CPU/内存是否稳定(不持续增长)
# - 没有崩溃或错误日志

# 使用 Ctrl+C 退出
```

### 5. 日志分析

```bash
# 统计解码器共享情况
echo "New decoders created: $(grep -c 'Acquired shared decoder' test.log)"
echo "Decoder reuses: $(grep -c 'Reusing existing decoder' test.log)"
echo "Final ref count: $(grep 'ref_count:' test.log | tail -1)"
```

## ✅ 通过标准

- [ ] 日志显示解码器共享(看到 "Reusing existing decoder")
- [ ] CPU 使用率明显降低(目测 <20%)
- [ ] 内存使用合理(100-150MB)
- [ ] 画面显示正常
- [ ] 运行稳定,无崩溃

## 📝 快速测试报告

```
Date: 2025-10-23
Branch: m5-shared-decode
Displays: [数量]

✅/❌ Decoder sharing: [工作/不工作]
✅/❌ Performance: [改善/无改善]
✅/❌ Stability: [稳定/不稳定]

Notes:
[任何观察到的问题或备注]
```

## 🚀 下一步

**如果快速测试通过**:
1. ✅ 核心功能正常
2. 📊 (可选)运行完整性能测试: `./scripts/test_m5_performance.sh`
3. ✅ Issue #13 可以标记为完成
4. ➡️  继续 Issue #14

**如果发现问题**:
1. 📋 收集完整日志: `RUST_LOG=trace ./target/release/wayvid &> debug.log`
2. 🔍 检查错误信息
3. 🐛 修复问题后重新测试

---

**预计时间**: 10分钟 ⏱️
**难度**: ⭐ 简单
