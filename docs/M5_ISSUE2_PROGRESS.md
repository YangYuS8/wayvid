# Issue #2 进展报告 (Part 1)

**Issue**: #2 - Advanced Multi-Monitor Features  
**日期**: 2025-10-24  
**分支**: `m5-multi-monitor`  
**状态**: 🚧 进行中 (23% complete)

---

## ✅ 已完成功能

### 1. 输出名称模式匹配 (3h)

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

## ⏳ 待实现功能

### 2. 输出优先级/Fallback (3h)

**目标**: 当多个模式匹配时,支持优先级排序

**设计**:
```yaml
per_output:
  "HDMI-*":
    priority: 1
    source: ...
  
  "*":
    priority: 99  # 最低优先级 fallback
    source: ...
```

### 3. wayvid-ctl 动态源切换 (3h)

**目标**: 运行时切换输出源

**命令**:
```bash
# 切换特定输出的源
wayvid-ctl set-output-source eDP-1 file:///new-video.mp4

# 切换所有输出
wayvid-ctl set-output-source --all file:///new-video.mp4
```

**IPC协议扩展**:
```rust
pub enum IpcCommand {
    // ...existing commands...
    SetOutputSource { output: String, source: VideoSource },
}
```

### 4. 配置Schema文档化 (1h)

更新文档说明:
- Pattern matching语法
- 优先级规则
- 示例配置

### 5. 多显示器测试 (3h)

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
| 优先级/Fallback | 3h | 0h | ⏳ |
| wayvid-ctl命令 | 3h | 0h | ⏳ |
| Schema文档 | 1h | 0h | ⏳ |
| 多显示器测试 | 3h | 0h | ⏳ |
| **总计** | **13h** | **3h** | **23%** |

---

## 🧪 测试结果

**单元测试**: 32/32 通过 ✅
- Pattern模块: 6个测试
- Config集成: 3个测试
- 其他模块: 23个测试

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

1. **明天继续**: 实现优先级/fallback逻辑
2. **然后**: 添加wayvid-ctl命令支持
3. **最后**: 文档和多显示器测试

**预计完成时间**: 明天下午 (还需10小时)

---

## 🎯 成功标准进度

- [x] 添加输出名称模式匹配
- [ ] 支持不同WE项目 per output
- [ ] 实现优先级/fallback
- [ ] 添加 wayvid-ctl set-output-source
- [ ] 更新配置schema
- [ ] 多显示器测试通过

**当前**: 1/6 完成

---

**Author**: YangYuS8  
**Branch**: m5-multi-monitor  
**Commit**: 5236d9c
