# M5 Shared Decode Context - Testing Guide

## 🎯 测试目标

验证 Issue #13 的核心功能:
1. ✅ 多显示器环境下解码器共享正常工作
2. 📈 CPU 使用率降低 60% (目标: 30% → 12%)
3. 💾 内存占用降低 73% (目标: 380MB → 100MB)

## 📋 测试前准备

### 1. 硬件要求
- **最低**: 2个显示器(或使用虚拟显示器)
- **推荐**: 3个显示器(以测试最佳效果)

### 2. 软件环境
```bash
# 确保在正确的分支
git checkout m5-shared-decode

# 编译 release 版本(性能测试需要优化版本)
cargo build --release --features video-mpv

# 准备测试视频
# 建议: 1080p 或 4K 视频,至少30秒长度
cp /path/to/your/test-video.mp4 ~/test.mp4
```

### 3. 测试配置
创建测试配置文件 `~/.config/wayvid/test-config.toml`:

```toml
[video]
source = { file = "/home/yangyus8/test.mp4" }
loop_playback = true

[video.hwdec]
mode = "auto"  # 或 "force" 如果有GPU

[render]
layout = "contain"
```

## 🧪 测试步骤

### Phase 1: 基线测试 (v0.3.0)

```bash
# 1. 切换到 main 分支
git checkout main
cargo build --release --features video-mpv

# 2. 启动 wayvid
WAYLAND_DISPLAY=wayland-1 ./target/release/wayvid \
  --config ~/.config/wayvid/test-config.toml \
  --log-level info

# 3. 记录性能数据(运行1分钟后)
# - 打开 htop 或 top,记录 CPU 使用率
# - 记录内存使用(RES 列)
# - 截图保存
```

**基线数据记录**:
```
Date: 2025-10-23
Branch: main (v0.3.0)
Displays: [数量]
Video: [分辨率] @ [帧率]

CPU Usage: _____% (每个进程)
Total CPU: _____% (所有进程总和)
Memory (RES): _____MB (每个进程)
Total Memory: _____MB (所有进程总和)
```

### Phase 2: 新版本测试 (m5-shared-decode)

```bash
# 1. 切换到开发分支
git checkout m5-shared-decode
cargo build --release --features video-mpv

# 2. 启动 wayvid(增加调试日志)
RUST_LOG=wayvid::video::shared_decode=debug,info \
  WAYLAND_DISPLAY=wayland-1 ./target/release/wayvid \
  --config ~/.config/wayvid/test-config.toml

# 3. 观察日志输出,确认共享工作正常
# 应该看到类似:
# ✅ Acquired shared decoder for source: file=/home/yangyus8/test.mp4
# ♻️ Reusing existing decoder (ref_count: 2)
# ♻️ Reusing existing decoder (ref_count: 3)
# 📊 Decoder stats: consumers=3, frames=1234

# 4. 记录性能数据(同样运行1分钟后)
```

**新版本数据记录**:
```
Date: 2025-10-23
Branch: m5-shared-decode
Displays: [数量]
Video: [分辨率] @ [帧率]

CPU Usage: _____% (主进程)
Memory (RES): _____MB (主进程)

Decoder Sharing: [✅/❌]
Ref Count: [数量]
```

### Phase 3: 性能对比

使用自动化脚本测量:

```bash
# 运行性能测试脚本
./scripts/test_m5_performance.sh
```

## 🔍 验证检查点

### 1. 解码器共享验证

在日志中查找:
```
✅ 第一个显示器初始化时应该看到:
   "Acquired shared decoder for source: file=..."
   "ref_count: 1"

✅ 后续显示器初始化时应该看到:
   "Reusing existing decoder"
   "ref_count: 2", "ref_count: 3", ...

❌ 如果看到多次 "Acquired shared decoder" 对相同的源,
   说明共享失败!
```

### 2. 引用计数验证

关闭一个显示器后:
```bash
# 使用 wlr-randr 或系统设置禁用一个显示器
wlr-randr --output HDMI-A-1 --off

# 观察日志:
# 应该看到: "Released decoder, new ref_count: 2"
# 不应该看到: "Cleanup decoder" (除非是最后一个)
```

### 3. 内存泄漏检查

```bash
# 长时间运行测试(30分钟)
# 每5分钟记录一次内存使用

# 如果内存持续增长 -> 可能有泄漏
# 如果内存稳定 -> 正常
```

## 📊 预期结果

### CPU 使用率 (3个显示器,1080p@30fps)

| 版本 | 每进程 | 总计 | 改善 |
|------|--------|------|------|
| v0.3.0 | 10% | 30% | - |
| m5 (目标) | 12% | 12% | **60%** ↓ |

### 内存占用 (3个显示器)

| 版本 | 每进程 | 总计 | 改善 |
|------|--------|------|------|
| v0.3.0 | 127MB | 381MB | - |
| m5 (目标) | 100MB | 100MB | **73%** ↓ |

### 解码器实例

| 版本 | 实例数 |
|------|--------|
| v0.3.0 | 3个(每输出一个) |
| m5 | 1个(共享) |

## 🐛 故障排查

### 问题 1: 编译失败
```bash
# 确保依赖最新
cargo clean
cargo update
cargo build --release --features video-mpv
```

### 问题 2: 没有看到共享日志
```bash
# 增加日志级别
RUST_LOG=wayvid::video::shared_decode=trace,debug ./target/release/wayvid
```

### 问题 3: 性能没有改善
可能原因:
- 测试视频太简单(尝试4K视频)
- GPU加速未启用(检查 hwdec 设置)
- 只有1个显示器(至少需要2个才能看到效果)

### 问题 4: 画面异常
可能原因:
- OpenGL 上下文共享问题
- 线程竞争(检查日志是否有错误)
- 解码器状态不一致

如果遇到,请收集:
```bash
# 完整日志
RUST_LOG=trace ./target/release/wayvid &> test.log

# 系统信息
uname -a
glxinfo | grep "OpenGL"
wlr-randr  # Wayland 显示信息
```

## ✅ 测试完成标准

- [ ] 日志确认解码器共享正常
- [ ] CPU 使用率降低 ≥50%
- [ ] 内存使用降低 ≥60%
- [ ] 画面正常显示(无黑屏/花屏)
- [ ] 长时间运行无崩溃(30分钟)
- [ ] 热插拔测试正常

## 📝 测试报告模板

```markdown
# M5 Shared Decode Context - Test Report

**Date**: 2025-10-23
**Tester**: [Your Name]
**Hardware**: [CPU/GPU/Displays]
**Environment**: [Wayland Compositor]

## Test Configuration
- Displays: 3 (1920x1080 @ 60Hz)
- Video: test.mp4 (1080p @ 30fps, H.264)
- Duration: 60 seconds each

## Results

### Decoder Sharing
- [✅/❌] Shared decoder confirmed in logs
- Ref count: 3 (as expected)

### Performance (v0.3.0 → m5-shared-decode)
- CPU: 30% → 12% (**60%** improvement ✅)
- Memory: 381MB → 100MB (**73.8%** improvement ✅)
- Decoder instances: 3 → 1 ✅

### Stability
- [✅/❌] No crashes in 30min run
- [✅/❌] Hot-plug working
- [✅/❌] No memory leaks

## Issues Found
1. [None / Issue description]

## Conclusion
[PASS / FAIL] - Ready for merge / Needs fixes
```

## 🚀 下一步

测试通过后:
1. 填写测试报告
2. 更新 `docs/SHARED_DECODE.md` 添加实测数据
3. 提交到 GitHub
4. 关闭 Issue #13
5. 开始 Issue #14 (Memory Optimization)
