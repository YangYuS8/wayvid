# Issue #2 进展报告 (Part 3)

**Issue**: #2 - Advanced Multi-Monitor Features  
**日期**: 2025-01-24  
**分支**: `m5-multi-monitor`  
**状态**: 🚧 进行中 (85% complete)

---

## ✅ 已完成功能

### 1. 输出名称模式匹配 (3h) ✅

**实现**:
- 新增 `src/config/pattern.rs` 模块
- 支持glob风格模式:
  - `*` - 匹配任意字符序列
  - `?` - 匹配任意单个字符
  - 精确匹配 (无通配符)

**匹配优先级**:
1. 精确匹配 (最高优先级)
2. 具体模式 (通配符越少越优先)
3. 长模式 (相同通配符数时,更长的优先)

**API**:
```rust
use wayvid::config::pattern::{matches_pattern, find_best_match};

// 检查是否匹配
assert!(matches_pattern("HDMI-A-1", "HDMI-*"));

// 找到最佳匹配
let patterns = vec!["HDMI-*", "HDMI-A-*", "HDMI-A-1"];
let best = find_best_match("HDMI-A-1", &patterns);
assert_eq!(best, Some("HDMI-A-1")); // 精确匹配
```

**配置示例**:
```yaml
source:
  type: File
  path: "/default.mp4"

per_output:
  # 精确匹配
  "eDP-1":
    layout: Fill
    source:
      type: File
      path: "/laptop-video.mp4"
  
  # HDMI显示器使用模式匹配
  "HDMI-*":
    layout: Contain
    source:
      type: File
      path: "/external-video.mp4"
  
  # DP显示器
  "DP-?":
    layout: Stretch
```

**测试覆盖**:
- 6个pattern模块测试
- 3个config集成测试
- 所有边界情况覆盖

---

### 2. 输出优先级/Fallback (2h) ✅

**实现**:
- 在 `OutputConfig` 添加 `priority` 字段 (默认: 50)
- 精确匹配总是优先级0 (最高)
- 修改 `Config::for_output()` 使用优先级排序

**优先级算法**:
```rust
score = if exact_match {
    0  // 总是最高优先级
} else {
    priority × 10000 + wildcards × 1000 - length
}
```

**配置示例**:
```yaml
per_output:
  # 精确匹配 - 总是最高优先级(隐式priority: 0)
  "eDP-1":
    source:
      type: File
      path: "/laptop.mp4"

  # 特定HDMI-A显示器 - 高优先级
  "HDMI-A-*":
    priority: 5
    source:
      type: File
      path: "/hdmi-a.mp4"
  
  # 所有其他HDMI - 较低优先级
  "HDMI-*":
    priority: 10
    source:
      type: File
      path: "/hdmi-generic.mp4"
  
  # 通配fallback - 最低优先级
  "*":
    priority: 99
    source:
      type: File
      path: "/fallback.mp4"
```

**测试覆盖**:
- 4个新priority测试
- 验证exact优先级
- 验证priority排序
- 验证fallback行为

---

### 3. wayvid-ctl 动态源切换 (3h) ✅

**实现**:
- 修改 `IpcCommand::SwitchSource` 使用 `VideoSource` 类型
- 更新 `handle_switch_source_command()` 处理所有source类型
- 添加 `parse_video_source()` CLI辅助函数

**支持的Source格式**:
```bash
# 本地文件 (两种格式)
wayvid-ctl switch -o eDP-1 file:///home/user/video.mp4
wayvid-ctl switch -o eDP-1 /home/user/video.mp4

# HTTP/HTTPS流
wayvid-ctl switch -o HDMI-A-1 https://example.com/stream.m3u8

# RTSP流
wayvid-ctl switch -o DP-1 rtsp://camera.local/stream

# 管道输入 (stdin)
wayvid-ctl switch -o eDP-1 pipe://
cat video.mp4 | wayvid
```

**其他命令** (已存在):
```bash
wayvid-ctl status          # 获取状态
wayvid-ctl pause -o eDP-1  # 暂停播放
wayvid-ctl resume -o eDP-1 # 恢复播放
wayvid-ctl seek -o eDP-1 30.0  # 跳转到30秒
wayvid-ctl rate -o eDP-1 1.5   # 1.5倍速
wayvid-ctl volume -o eDP-1 0.8 # 80%音量
wayvid-ctl mute -o eDP-1       # 切换静音
wayvid-ctl layout -o eDP-1 cover  # 设置布局
wayvid-ctl reload          # 重新加载配置
wayvid-ctl quit            # 退出守护进程
```

**测试覆盖**:
- 编译通过 ✅
- 所有35个单元测试通过 ✅

---

### 4. 配置Schema文档化 (2h) ✅

**创建文档**:
- `docs/MULTI_MONITOR_EXAMPLES.md` - 完整使用指南

**内容包括**:
- 模式匹配语法和示例
- 优先级系统详解
- 运行时控制命令大全
- 5个常见场景配置示例:
  1. 笔记本+外接显示器
  2. 多显示器工作站
  3. 优先级fallback
  4. 开发vs生产环境
  5. 热插拔处理
- 最佳实践
- 故障排查

---

## ⏳ 待实现功能

### 5. 多显示器测试 (2h)

在真实硬件上测试:
- 2+ 显示器场景
- 热插拔
- 模式匹配正确性
- 性能验证

---

## 📊 进度统计

| 任务 | 预算 | 已用 | 状态 |
|------|------|------|------|
| Pattern Matching | 3h | 3h | ✅ |
| 优先级/Fallback | 2h | 2h | ✅ |
| wayvid-ctl命令 | 3h | 3h | ✅ |
| Schema文档 | 2h | 2h | ✅ |
| 多显示器测试 | 2h | 0h | ⏳ |
| **总计** | **12h** | **10h** | **85%** |

---

## 🧪 测试结果

**单元测试**: 35/35 通过 ✅
- Pattern模块: 6个测试
- Config集成: 6个测试 (含4个priority测试)
- Protocol: 1个更新测试
- 其他模块: 22个测试

**Clippy**: 无警告 ✅  
**Format**: 通过 ✅

---

## 📝 技术笔记

### Pattern Matching算法

使用递归回溯算法实现glob匹配:
- `*` 匹配通过尝试所有可能的位置
- `?` 匹配精确一个字符
- 精确字符匹配

**复杂度**: O(n×m) 其中 n=name长度, m=pattern长度

### 最佳匹配评分

```rust
score = if exact_match {
    0  // 最高优先级
} else {
    wildcards_count × 1000 - pattern_length
}
```

- 通配符越少,分数越低(越优先)
- 相同通配符数时,越长越优先
- 精确匹配总是最优

---

## 🔄 下一步

1. **多显示器测试** (2h):
   - 在真实多显示器环境测试
   - 验证pattern匹配正确性
   - 验证热插拔行为
   - 性能测试

2. **创建PR并合并**:
   - 完整的PR描述
   - 所有测试通过
   - 文档完整

**预计完成时间**: 今天稍后 (还需2小时测试)

---

## 🎯 成功标准进度

- [x] 添加输出名称模式匹配
- [x] 支持不同源 per output (通过pattern+priority)
- [x] 实现优先级/fallback
- [x] 添加 wayvid-ctl switch 命令 (已有,已更新为VideoSource)
- [x] 更新配置schema文档 (MULTI_MONITOR_EXAMPLES.md)
- [ ] 多显示器测试通过

**当前**: 5/6 完成 (85%)

---

## 📂 修改的文件

**新增文件**:
- `src/config/pattern.rs` - 模式匹配逻辑
- `docs/MULTI_MONITOR_EXAMPLES.md` - 完整使用指南

**修改文件**:
- `src/config/types.rs` - 添加priority字段, 更新for_output()
- `src/config/mod.rs` - 导出pattern模块
- `src/ctl/protocol.rs` - SwitchSource使用VideoSource
- `src/backend/wayland/app.rs` - 更新handle_switch_source_command()
- `src/bin/wayvid-ctl.rs` - 添加parse_video_source()
- `docs/M5_ISSUE2_PROGRESS.md` - 本文档

---

**Author**: YangYuS8  
**Branch**: m5-multi-monitor  
**Commits**: 5236d9c (pattern), 32b670c (priority), <pending> (IPC)
