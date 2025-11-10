# M6 开发启动说明

**日期**: 2025-11-10  
**状态**: 已启动 🚀  
**目标**: Niri + Workshop 生态系统集成

---

## 📋 快速概览

### 项目定位
wayvid 现在专注于成为 **Niri 滚动平铺混成器**和 **Noctalia Shell** 的标准视频壁纸解决方案，同时提供无缝的 **Steam Workshop 集成**。

### 主要目标发行版
- **Arch Linux** (主要)
- EndeavourOS
- Manjaro

### 核心用户群
1. Niri + Arch Linux 用户
2. Wallpaper Engine 迁移用户
3. Noctalia Shell 早期采用者

---

## 🎯 M6 里程碑概览

### Phase 0: 核心集成 (Week 1) ⏳
**状态**: 待开始  
**工时**: 26h

- **Issue #23**: Steam Workshop Integration (12h, P0)
  - Steam 库发现
  - Workshop 项目扫描
  - `wayvid workshop list/import` 命令
  
- **Issue #24**: Niri-Specific Optimizations (14h, P0)
  - Niri 兼容性测试
  - 工作区感知优化
  - 滚动场景性能调优

### Phase 1: 用户体验 (Week 2)
**状态**: 规划中  
**工时**: 32h

- **Issue #25**: Arch Linux Packaging (10h, P1)
- **Issue #3**: Playlist Support (14h, P1)
- **Issue #6**: Configuration Validator (8h, P2)

### Phase 2: Noctalia 准备 (Week 3)
**状态**: 规划中  
**工时**: 34h

- Noctalia Shell Integration Preparation (16h, P1)
- Issue #7: Interactive Setup Wizard (10h, P2)
- Issue #8: Diagnostic Tools (8h, P2)

### Phase 3: 生态完善 (Week 4)
**状态**: 规划中  
**工时**: 30h

- 文档工作 (12h)
- 测试和 QA (10h)
- 社区建设 (8h)

---

## 📦 已创建的 Issues

### P0 (Critical)
1. **#23** - [M6-P0] Steam Workshop Integration
   - 标签: `workshop`, `m6`, `enhancement`
   - 工时: 12h
   - [查看 Issue](https://github.com/YangYuS8/wayvid/issues/23)

2. **#24** - [M6-P0] Niri-Specific Optimizations
   - 标签: `niri`, `m6`, `enhancement`
   - 工时: 14h
   - [查看 Issue](https://github.com/YangYuS8/wayvid/issues/24)

### P1 (High)
3. **#25** - [M6-P1] Arch Linux Packaging Improvements
   - 标签: `m6`, `enhancement`, `distribution`
   - 工时: 10h
   - [查看 Issue](https://github.com/YangYuS8/wayvid/issues/25)

---

## 🏗️ 技术栈更新

### 新增模块
```
src/
├── we/
│   ├── steam.rs        # Steam 库管理
│   └── workshop.rs     # Workshop API
├── backend/
│   └── wayland/
│       └── niri.rs     # Niri 特定逻辑
└── dbus/               # D-Bus 接口 (Phase 2)
    └── wallpaper_manager.rs
```

### 新增依赖
```toml
keyvalues-parser = "0.2"  # VDF 解析
dirs = "5.0"              # 跨平台目录
zbus = "4.0"              # D-Bus (可选)
```

---

## 🚀 快速开始开发

### 1. 环境准备
```bash
# 确保在 Niri + Arch Linux 环境
# 已有 Steam 和 Wallpaper Engine

# 更新依赖
cargo update

# 运行测试
cargo test
```

### 2. 开始 Issue #23 (Workshop Integration)
```bash
# 创建功能分支
git checkout -b m6-workshop

# 创建新文件
touch src/we/steam.rs src/we/workshop.rs

# 开始编码
$EDITOR src/we/steam.rs
```

### 3. 开发流程
1. 选择一个 Issue
2. 创建功能分支 `m6-feature-name`
3. 编写代码 + 测试
4. 运行 `cargo fmt && cargo clippy`
5. 提交 PR 并关联 Issue
6. 等待 CI 通过
7. 合并到 main

---

## 📚 关键文档

### 必读
- [M6 路线图](M6_ROADMAP.md) - 完整规划
- [WE 格式文档](WE_FORMAT.md) - 理解 Workshop 项目结构
- [Niri Noctalia 路线图](NIRI_NOCTALIA_ROADMAP.md) - 长期愿景

### 参考
- [共享解码文档](SHARED_DECODE.md) - M5 架构参考
- [IPC 文档](IPC.md) - 现有 IPC 实现
- [多显示器示例](MULTI_MONITOR_EXAMPLES.md) - 配置参考

---

## 🧪 测试环境需求

### 推荐配置
- **OS**: Arch Linux (最新)
- **Compositor**: Niri (git)
- **Steam**: 已安装
- **WE**: 至少订阅 3-5 个视频壁纸
- **硬件**: 支持 VA-API 的 GPU

### 可选配置
- 多显示器设置
- Hyprland (对比测试)
- 虚拟机 (兼容性测试)

---

## 📊 进度跟踪

### 本周目标 (Week 1)
- [ ] Issue #23 完成 50% (Steam 发现 + VDF 解析)
- [ ] Issue #24 开始 (Niri 兼容性测试)
- [ ] 创建基础测试套件

### 本月目标
- [ ] Phase 0 完成 (Workshop + Niri 核心)
- [ ] Phase 1 开始 (播放列表)
- [ ] v0.5.0-alpha 发布

---

## 🎯 成功指标

### 技术指标
- Workshop 扫描速度 < 1s (100+ 项目)
- Niri 工作区切换流畅 (>55 FPS)
- 内存占用合理 (<300MB, 4 工作区)

### 社区指标
- AUR 包周下载量 > 100
- GitHub Stars > 50
- 社区正面反馈 > 90%

---

## 💬 沟通渠道

### 技术讨论
- GitHub Issues: Bug 和功能请求
- GitHub Discussions: 设计讨论
- PR Reviews: 代码审查

### 社区交流
- Niri Discord: Niri 用户反馈
- Reddit r/unixporn: 展示和推广
- Arch Linux Forums: 支持和文档

---

## 🔧 开发工具

### 代码质量
```bash
# 格式化
cargo fmt --all

# Lint
cargo clippy --all-features -- -D warnings

# 测试
cargo test --all-features

# 覆盖率
cargo tarpaulin --out Html
```

### 调试
```bash
# 启用详细日志
RUST_LOG=debug wayvid run

# 性能分析
perf record -g wayvid run
perf report

# 内存检查
valgrind --leak-check=full wayvid run
```

---

## 📝 提交规范

### Commit 消息格式
```
<type>(<scope>): <subject>

<body>

<footer>
```

### 类型 (type)
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

### 示例
```
feat(workshop): Add Steam library discovery

- Implement SteamLibrary struct
- Parse libraryfolders.vdf
- Find Workshop items by app ID
- Add unit tests

Closes #23
```

---

## 🚨 注意事项

### 兼容性
- 保持与 Hyprland 的兼容性
- 不破坏现有配置文件
- 保持向后兼容 API

### 性能
- Workshop 扫描要快 (<1s)
- Niri 优化不影响其他混成器
- 内存占用可控

### 文档
- 每个新功能都要有文档
- 更新 README 和 QUICKSTART
- 添加配置示例

---

## 🎉 里程碑庆祝

### Phase 0 完成
- 发布 v0.5.0-alpha
- 博客文章
- 社区公告

### v0.5.0 正式发布
- Release Notes
- Reddit/HN 发布
- Niri Discord 公告
- 视频演示

---

## 📞 联系方式

### 项目维护者
- GitHub: [@YangYuS8](https://github.com/YangYuS8)
- Email: YangYuS8@users.noreply.github.com

### 报告问题
- GitHub Issues: https://github.com/YangYuS8/wayvid/issues
- 请附带详细信息和日志

---

## ✅ Checklist

### 立即行动
- [x] 创建 M6 路线图文档
- [x] 创建 Issues #23, #24, #25
- [x] 更新 README
- [x] 创建开发标签
- [ ] 开始 Issue #23 实现

### 本周
- [ ] Steam 库发现功能
- [ ] Workshop 扫描原型
- [ ] Niri 兼容性测试
- [ ] 创建测试脚本

### 本月
- [ ] Phase 0 完成
- [ ] Phase 1 开始
- [ ] Alpha 版本发布

---

**准备好了吗？让我们开始构建 Niri 生态的标准壁纸解决方案！** 🚀

---

_Created: 2025-11-10_  
_Status: Active Development_  
_Next: Implement Issue #23_
