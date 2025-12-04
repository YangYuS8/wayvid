# v0.5 GUI-First 重构 - 任务清单

## Phase 1: 项目结构重组 (Week 1) ✅ COMPLETED

### 1.1 Workspace 初始化
- [x] 创建 workspace Cargo.toml
- [x] 迁移 src/core → crates/wayvid-core
- [x] 迁移 src/video + src/backend → crates/wayvid-engine
- [x] 创建 crates/wayvid-library (新模块)
- [x] 创建 crates/wayvid-gui (iced 框架)
- [x] 增强 crates/wayvid-ctl (IPC 支持)

### 1.2 依赖整理
- [x] 整理各 crate 依赖关系
- [x] 统一版本号管理 (workspace.dependencies)
- [x] 更新 GUI 框架从 egui 到 iced

### 1.3 构建验证
- [x] 确保所有 crate 独立编译
- [x] 验证 34+ 单元测试通过
- [ ] 更新 CI 配置

### 1.4 wayvid-core (已完成)
- [x] types.rs: WallpaperItem, WallpaperType, WallpaperMetadata
- [x] config/mod.rs: Config 结构体
- [x] config/types.rs: OutputConfig, VideoSource
- [x] config/pattern.rs: 输出名称模式匹配
- [x] layout.rs: ScaleMode, compute_layout
- [x] power.rs: 电源管理检测

### 1.5 wayvid-engine (已完成)
- [x] wayland/output.rs: OutputManager, OutputInfo
- [x] frame_timing.rs: FrameTiming 帧率控制
- [x] 模块导出和 lib.rs

### 1.6 wayvid-library (已完成)
- [x] database.rs: SQLite 壁纸数据库
- [x] scanner.rs: 文件夹扫描器
- [x] thumbnail.rs: 缩略图生成
- [x] 9 个单元测试

### 1.7 wayvid-gui (已完成)
- [x] iced 框架集成
- [x] 四个视图: Library, Folders, Settings, About
- [x] 侧边栏导航
- [x] IPC 客户端存根

### 1.8 wayvid-ctl (已完成)
- [x] 完整命令套件
- [x] Unix socket IPC 客户端
- [x] JSON 协议支持
- [x] 3 个单元测试

## Phase 2: 壁纸库模块深化 (Week 2) ✅ COMPLETED

### 2.1 数据库增强
- [x] 添加全文搜索索引
- [x] 实现标签系统
- [x] 添加收藏功能

### 2.2 扫描器优化
- [x] 实现 rayon 并行扫描
- [x] 文件变更监控 (notify crate)
- [x] 增量扫描优化

### 2.3 缩略图系统
- [x] WebP 输出格式
- [x] 缓存目录结构 (~/.cache/wayvid/thumbnails/)
- [x] 后台线程生成
- [x] GIF 动画预览

### 2.4 Workshop 集成
- [x] 迁移 Steam Workshop 扫描逻辑
- [x] 自动检测 Steam 路径
- [x] 解析 project.json 元数据

## Phase 3: GUI 重构 (Week 3-4) ✅ COMPLETED

### 3.1 模块拆分
- [x] 创建 views/ 目录结构
- [x] 拆分 library view (壁纸网格)
- [x] 拆分 monitors view (显示器管理)
- [x] 拆分 settings view (设置页面)

### 3.2 组件化
- [x] 创建 widgets/ 目录
- [x] WallpaperCard 组件 (带缩略图)
- [x] MonitorSelector 组件
- [x] ThumbnailImage 组件 (异步加载)

### 3.3 异步加载
- [x] 缩略图异步加载 (channel + texture)
- [x] 虚拟滚动 (只渲染可见区域)
- [x] 加载状态指示器

### 3.4 内置服务
- [x] 将 daemon 逻辑内置到 GUI
- [x] 窗口关闭时最小化到托盘
- [x] 后台壁纸渲染

### 3.5 托盘支持
- [x] 添加系统托盘图标
- [x] 托盘菜单 (显示/暂停/退出)
- [x] 托盘通知

## Phase 4: 配置自动化 (Week 4) ✅ COMPLETED

### 4.1 AppSettings
- [x] 定义 settings.yaml 结构
- [x] 自动保存 (debounce)
- [x] 默认值处理

### 4.2 开机自启动
- [x] 检测 XDG autostart
- [x] 生成 .desktop 文件
- [x] GUI 开关控制

### 4.3 电源管理
- [x] 全屏应用检测 (暂停壁纸)
- [x] 电池模式检测
- [x] FPS 限制选项

## Phase 5: CLI 精简 (Week 5) ✅ COMPLETED

### 5.1 wayvid-ctl 重写
- [x] 保留基础命令: apply, pause, resume, status
- [x] 移除复杂命令: run, daemon, import, workshop
- [x] JSON 输出格式

### 5.2 IPC 更新
- [x] 简化 IPC 协议
- [x] 添加 GetLibrary 命令 (可选)

### 5.3 旧代码清理
- [x] 共享 IPC 协议到 wayvid-core
- [x] 移除未使用的 CLI 子命令
- [x] 更新帮助文档

## Phase 6: 测试与打包 (Week 5-6)

### 6.1 测试
- [ ] wayvid-core 单元测试
- [ ] wayvid-library 单元测试
- [ ] GUI 集成测试 (手动)
- [ ] CLI 集成测试

### 6.2 文档更新
- [ ] 更新 README.md
- [ ] 更新 docs/ mdbook
- [ ] 更新安装说明

### 6.3 打包
- [ ] 更新 AUR PKGBUILD
- [ ] 更新 Nix flake
- [ ] 验证 AppImage 构建

### 6.4 发布
- [ ] 更新 CHANGELOG.md
- [ ] 创建 v0.5.0 release
- [ ] 社区公告

## 里程碑

| 里程碑 | 目标日期 | 状态 |
|-------|---------|------|
| Phase 1: 项目结构 | Week 1 | ✅ 已完成 |
| Phase 2: 壁纸库 | Week 2 | ✅ 已完成 |
| Phase 3: GUI 重构 | Week 4 | ✅ 已完成 |
| Phase 4: 配置自动化 | Week 4 | ✅ 已完成 |
| Phase 5: CLI 精简 | Week 5 | ✅ 已完成 |
| Phase 6: 发布 | Week 6 | 进行中 |

## 注意事项

1. **保持向后兼容**: 现有 config.yaml 格式继续支持
2. **渐进式迁移**: 每个 phase 完成后确保可用
3. **性能监控**: 重构过程中持续关注内存/CPU 使用
4. **用户反馈**: 可发布 alpha 版本收集反馈
