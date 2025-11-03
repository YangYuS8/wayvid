# wayvid 文档索引

> **最后更新**: 2025-11-03  
> **文档版本**: 2.0

欢迎来到 wayvid 文档中心！本索引帮助您快速找到所需的文档。

---

## 📚 用户文档

### 快速开始
- **[QUICKSTART.md](QUICKSTART.md)** - 5分钟快速开始指南
  - 安装步骤
  - 基础配置
  - 常见问题

### 功能指南
- **[HDR_USER_GUIDE.md](HDR_USER_GUIDE.md)** - HDR 支持完整指南
  - HDR 检测和配置
  - 5种色调映射算法
  - 内容优化建议
  - 故障排查

- **[MULTI_MONITOR_EXAMPLES.md](MULTI_MONITOR_EXAMPLES.md)** - 多显示器配置示例
  - Per-output 配置
  - 输出匹配模式
  - 热插拔处理

### 技术参考
- **[IPC.md](IPC.md)** - IPC 命令行接口
  - 命令列表和参数
  - 使用示例
  - 集成指南

- **[VIDEO_SOURCES.md](VIDEO_SOURCES.md)** - 视频源配置
  - 支持的格式
  - 布局模式
  - 性能优化

- **[WE_FORMAT.md](WE_FORMAT.md)** - Wallpaper Engine 格式兼容
  - 项目结构
  - 参数映射
  - 转换工具

### 实现细节
- **[SHARED_DECODE.md](SHARED_DECODE.md)** - 共享解码架构
  - 实现原理
  - 性能收益
  - 使用场景

- **[HDR_WAYLAND_STATUS.md](HDR_WAYLAND_STATUS.md)** - Wayland HDR 支持现状
  - 协议支持情况
  - 合成器兼容性
  - 未来展望

### 技术背景
- **[HDR_WAYLAND_STATUS.md](HDR_WAYLAND_STATUS.md)** - Wayland HDR 支持现状
  - 协议支持情况
  - 合成器兼容性
  - 未来展望

---

## 🔧 开发者文档

### 项目根目录
- **[README.md](../README.md)** - 项目主页和概览
- **[AI_PROMPT.md](../AI_PROMPT.md)** - 完整的 AI 开发提示词
  - 项目全貌
  - 开发计划
  - 技术深入解析
  - 迁移设备指南

- **[CHANGELOG.md](../CHANGELOG.md)** - 版本更新日志
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - 贡献指南

### 配置示例
- **[../examples/config.yaml](../examples/config.yaml)** - 基础配置示例
- **[../examples/hdr-config.yaml](../examples/hdr-config.yaml)** - HDR 配置示例
- **[../examples/multi-monitor.yaml](../examples/multi-monitor.yaml)** - 多显示器配置

---

## 📁 归档文档

历史文档和开发过程记录已移至 `docs/archive/` 目录：

### 里程碑归档
- **[archive/m1/](archive/m1/)** - M1 里程碑文档
  - 交付报告、测试清单、最终测试报告

- **[archive/m2/](archive/m2/)** - M2 里程碑文档
  - 多显示器实现计划、进度报告、阶段性报告

- **[archive/m3/](archive/m3/)** - M3 里程碑文档
  - Wallpaper Engine 兼容性交付报告

- **[archive/m4/](archive/m4/)** - M4 里程碑文档
  - 完成报告

- **[archive/m5/](archive/m5/)** - M5 里程碑文档
  - 性能优化文档 (共享解码、内存优化、懒加载、帧跳跃)
  - HDR 和多显示器实现进度
  - HDR 测试报告和实现总结
  - 测试指南和 Phase 完成报告

### 其他归档
- **[archive/releases/](archive/releases/)** - 历史版本发布说明
- **[archive/development/](archive/development/)** - 开发过程文档
  - 开发笔记、优化报告、PR 描述
  - 项目结构、速查表

---

## 🗂️ 文档结构总览

```
rustpaper/
├── README.md                    # 项目主页
├── AI_PROMPT.md                 # AI 开发提示词 ⭐
├── CHANGELOG.md                 # 版本更新日志
├── CONTRIBUTING.md              # 贡献指南
├── QUICKSTART.md                # 快速开始
│
├── docs/                        # 用户和技术文档
│   ├── README.md                # 本文件
│   ├── QUICKSTART.md            # 快速开始指南
│   │
│   ├── HDR_USER_GUIDE.md        # HDR 用户指南 ⭐
│   ├── HDR_WAYLAND_STATUS.md    # Wayland HDR 现状
│   │
│   ├── MULTI_MONITOR_EXAMPLES.md # 多显示器示例 ⭐
│   │
│   ├── IPC.md                   # IPC 命令参考 ⭐
│   ├── VIDEO_SOURCES.md         # 视频源配置
│   ├── WE_FORMAT.md             # WE 格式说明
│   ├── SHARED_DECODE.md         # 共享解码架构
│   │
│   └── archive/                 # 历史文档归档
│       ├── m1/                  # M1 里程碑
│       ├── m2/                  # M2 里程碑
│       ├── m3/                  # M3 里程碑
│       ├── m4/                  # M4 里程碑
│       ├── m5/                  # M5 里程碑
│       ├── releases/            # 发布说明
│       └── development/         # 开发文档
│
├── examples/                    # 配置示例
│   ├── config.yaml
│   ├── hdr-config.yaml
│   └── multi-monitor.yaml
│
└── scripts/                     # 测试和验证脚本
    ├── verify-hdr-implementation.sh
    ├── test-hdr-functionality.sh
    └── test-multi-monitor.sh
```

---

## 🔍 快速查找

### 我想要...

**开始使用 wayvid**
→ [QUICKSTART.md](QUICKSTART.md)

**配置 HDR 支持**
→ [HDR_USER_GUIDE.md](HDR_USER_GUIDE.md)

**设置多显示器**
→ [MULTI_MONITOR_EXAMPLES.md](MULTI_MONITOR_EXAMPLES.md)

**通过命令行控制**
→ [IPC.md](IPC.md)

**导入 Wallpaper Engine 壁纸**
→ [WE_FORMAT.md](WE_FORMAT.md)

**了解项目完整状态**
→ [AI_PROMPT.md](../AI_PROMPT.md)

**查看开发进展**
→ [CHANGELOG.md](../CHANGELOG.md)

**贡献代码**
→ [CONTRIBUTING.md](../CONTRIBUTING.md)

**排查问题**
→ [HDR_USER_GUIDE.md#troubleshooting](HDR_USER_GUIDE.md) + [AI_PROMPT.md#常见问题](../AI_PROMPT.md)

---

## 📝 文档维护

### 文档分类原则

**保留在 docs/ 的文档**:
- ✅ 用户常用指南和参考
- ✅ 核心功能说明
- ✅ 技术实现细节 (长期有效)
- ✅ 最新测试报告

**归档到 archive/ 的文档**:
- 📦 历史里程碑报告
- 📦 阶段性进度文档
- 📦 过时的测试报告
- 📦 开发过程笔记

### 更新文档

当文档更新时，请：
1. 更新文档本身的"最后更新"日期
2. 如果是重大更改，更新本 README.md
3. 考虑是否需要同步更新 AI_PROMPT.md

---

## 📞 获取帮助

- **GitHub Issues**: 报告 Bug 或提出功能请求
- **GitHub Discussions**: 提问和讨论
- **Email**: YangYuS8@163.com

---

**文档状态**: ✅ 已整理和归档 (2025-11-03)  
**总文档数**: 52 个文件 (9 个活跃 + 41 个归档 + 2 个索引)
