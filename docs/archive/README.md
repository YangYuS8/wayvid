# 文档归档说明

本目录包含 wayvid 项目的历史文档和开发过程记录。

## 📁 目录结构

```
archive/
├── m1/              # M1 里程碑 (基础功能)
├── m2/              # M2 里程碑 (多显示器)
├── m3/              # M3 里程碑 (Wallpaper Engine 兼容)
├── m4/              # M4 里程碑 (稳定性提升)
├── m5/              # M5 里程碑 (性能优化 + HDR/多显示器)
├── releases/        # 历史版本发布说明
└── development/     # 开发过程文档
```

## 📋 各目录内容

### M1 - 基础功能 (2025-10)
- 交付报告 (`M1_DELIVERY_REPORT.md`)
- 功能清单 (`M1_CHECKLIST.md`)
- 测试报告 (`M1_TEST_REPORT.md`, `M1_FINAL_TEST_SUCCESS.md`)

**关键成就**:
- 单输出视频播放
- Layer-shell 背景层实现
- 基本配置系统
- AppImage/AUR/Nix 打包

### M2 - 多显示器支持 (2025-10)
- 实施计划 (`M2_PLAN.md`)
- 进度跟踪 (`M2_PROGRESS.md`)
- 交付报告 (`M2_DELIVERY_REPORT.md`)
- 阶段报告 (`M2_PHASE2_TEST_REPORT.md`, `M2_PHASE4_REPORT.md`)

**关键成就**:
- 输出热插拔支持
- Per-output 配置覆盖
- 输出匹配模式 (exact/prefix/suffix/regex)
- 动态缩放和旋转处理

### M3 - Wallpaper Engine 兼容 (2025-10)
- 交付报告 (`M3_DELIVERY_REPORT.md`)

**关键成就**:
- WE 项目导入工具
- 视频类壁纸参数兼容
- 多种布局模式支持

### M4 - 稳定性提升 (2025-10)
- 完成报告 (`M4_COMPLETION.md`)

**关键成就**:
- CI/CD 完善
- 错误处理优化
- 日志系统改进
- 文档完善

### M5 - 性能与高级特性 (2025-10至今)

#### Phase 1: 性能优化 (P0) ✅
**文档**:
- `M5_PLAN.md` - M5 总体规划
- `M5_TODO.md` - 任务清单
- `M5_GITHUB_PROJECT.md` - GitHub Project 设置
- `M5_QUICKSTART.md` - M5 快速开始
- `M5_PROGRESS.md` - 总体进度

**性能优化文档**:
- `M5_MEMORY_TEST.md` / `M5_MEMORY_TEST_RESULTS.md` - 内存优化测试
- `M5_LAZY_INIT.md` - 懒加载实现
- `M5_FRAME_SKIP.md` - 智能帧跳跃
- `M5_PHASE1_COMPLETE.md` - Phase 1 完成报告
- `M5_QUICK_TEST.md` / `M5_TEST_GUIDE.md` - 测试指南

**关键成就**:
- Issue #13: 共享解码上下文 (CPU 使用率降低 60%+)
- Issue #14: 内存优化 (内存占用降低 40%)
- Issue #15: 懒加载初始化
- Issue #16: 智能帧跳跃

#### Phase 2: HDR 和多显示器 (P1) ✅
**文档**:
- `M5_ISSUE1_PLAN.md` - HDR 实现计划
- `M5_ISSUE1_PROGRESS.md` - HDR 实现进度
- `M5_ISSUE2_PROGRESS.md` - 多显示器进度
- `M5_ISSUE2_TESTING.md` / `M5_ISSUE2_TEST_REPORT.md` - 多显示器测试

**关键成就**:
- Issue #1: HDR 支持 (自动检测、5种色调映射、内容优化)
- Issue #2: 高级多显示器 (输出模式匹配、per-output 覆盖)

### Releases - 版本发布
- `RELEASE_NOTES_v0.3.0.md` - v0.3.0 发布说明
- `RELEASE_NOTES_SHORT.md` - 简短发布说明

### Development - 开发文档
- `DEV_NOTES.md` - 开发笔记
- `OPTIMIZATION_REPORT.md` - 优化报告
- `PROJECT_STRUCTURE.md` - 项目结构说明
- `CHEATSHEET.md` - 速查表
- `PR_DESCRIPTION_*.md` - PR 描述模板
- `SUMMARY.md` - 项目总结
- `test_report.md` - 测试报告

## 🔍 查找历史信息

### 按功能查找

**多显示器实现**
→ `m2/M2_PROGRESS.md`

**HDR 支持实现**
→ `m5/M5_ISSUE1_PROGRESS.md`

**性能优化过程**
→ `m5/M5_MEMORY_TEST_RESULTS.md`, `m5/M5_PHASE1_COMPLETE.md`

**测试方法**
→ `m1/M1_TEST_REPORT.md`, `m5/M5_TEST_GUIDE.md`

### 按时间查找

**2025-10-20 至 10-21**: M1 完成
**2025-10-21 至 10-22**: M2 开发和完成
**2025-10-22 至 10-23**: M3-M4 完成
**2025-10-23 至 10-25**: M5 Phase 1-2 完成
**2025-10-25 至今**: M5 Phase 3 进行中

## 📊 统计信息

- **总文档数**: 39 个文件
- **覆盖时间**: 2025-10-20 至 2025-11-03
- **主要里程碑**: 5 个 (M1-M5)
- **已完成 Issues**: 6 个 (M5 中的 #1, #2, #13-16)

## 📝 文档归档原则

**归档的文档**:
- ✅ 已完成里程碑的报告
- ✅ 阶段性进度文档
- ✅ 过时的测试报告
- ✅ 开发过程笔记
- ✅ PR 描述和计划文档

**保留在主文档区的**:
- ✅ 用户指南和教程
- ✅ 技术参考文档
- ✅ 最新功能文档
- ✅ 当前开发计划

## 🔄 文档维护

**归档时间**: 2025-11-03  
**归档理由**: 设备迁移前的文档整理，保留历史记录同时简化主文档结构  
**归档策略**: 按里程碑组织，保持清晰的时间线和功能线索

## 📞 获取帮助

如果您需要查找特定的历史信息但不确定在哪个文档中，请：
1. 查看本 README
2. 搜索 GitHub Issues 和 PR
3. 查看主项目的 `AI_PROMPT.md` (包含完整项目历史)

---

**归档版本**: v1.0  
**最后更新**: 2025-11-03
