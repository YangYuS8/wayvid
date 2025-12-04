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

## Phase 2: 壁纸库模块深化 (Week 2)

### 2.1 数据库增强
- [ ] 添加全文搜索索引
- [ ] 实现标签系统
- [ ] 添加收藏功能

### 2.2 扫描器优化
- [ ] 实现 rayon 并行扫描
- [ ] 文件变更监控 (notify crate)
- [ ] 增量扫描优化

### 2.3 缩略图系统
- [ ] WebP 输出格式
- [ ] 缓存目录结构 (~/.cache/wayvid/thumbnails/)
- [ ] 后台线程生成
- [ ] GIF 动画预览

### 2.4 Workshop 集成
- [ ] 迁移 Steam Workshop 扫描逻辑
- [ ] 自动检测 Steam 路径
- [ ] 解析 project.json 元数据

## Phase 3: GUI 重构 (Week 3-4)

### 3.1 模块拆分
- [ ] 创建 views/ 目录结构
- [ ] 拆分 library view (壁纸网格)
- [ ] 拆分 monitors view (显示器管理)
- [ ] 拆分 settings view (设置页面)

### 3.2 组件化
- [ ] 创建 widgets/ 目录
- [ ] WallpaperCard 组件 (带缩略图)
- [ ] MonitorSelector 组件
- [ ] ThumbnailImage 组件 (异步加载)

### 3.3 异步加载
- [ ] 缩略图异步加载 (channel + texture)
- [ ] 虚拟滚动 (只渲染可见区域)
- [ ] 加载状态指示器

### 3.4 内置服务
- [ ] 将 daemon 逻辑内置到 GUI
- [ ] 窗口关闭时最小化到托盘
- [ ] 后台壁纸渲染

### 3.5 托盘支持
- [ ] 添加系统托盘图标
- [ ] 托盘菜单 (显示/暂停/退出)
- [ ] 托盘通知

## Phase 4: 配置自动化 (Week 4)

### 4.1 AppSettings
- [ ] 定义 settings.yaml 结构
- [ ] 自动保存 (debounce)
- [ ] 默认值处理

### 4.2 开机自启动
- [ ] 检测 XDG autostart
- [ ] 生成 .desktop 文件
- [ ] GUI 开关控制

### 4.3 电源管理
- [ ] 全屏应用检测 (暂停壁纸)
- [ ] 电池模式检测
- [ ] FPS 限制选项

## Phase 5: CLI 精简 (Week 5)

### 5.1 wayvid-ctl 重写
- [ ] 保留基础命令: apply, pause, resume, status
- [ ] 移除复杂命令: run, daemon, import, workshop
- [ ] JSON 输出格式

### 5.2 IPC 更新
- [ ] 简化 IPC 协议
- [ ] 添加 GetLibrary 命令 (可选)

### 5.3 旧代码清理
- [ ] 移除 main.rs 中的复杂逻辑
- [ ] 移除未使用的 CLI 子命令
- [ ] 更新帮助文档

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
| Phase 1: 项目结构 | Week 1 | 进行中 |
| Phase 2: 壁纸库 | Week 2 | 未开始 |
| Phase 3: GUI 重构 | Week 4 | 未开始 |
| Phase 4: 配置自动化 | Week 4 | 未开始 |
| Phase 5: CLI 精简 | Week 5 | 未开始 |
| Phase 6: 发布 | Week 6 | 未开始 |

## 注意事项

1. **保持向后兼容**: 现有 config.yaml 格式继续支持
2. **渐进式迁移**: 每个 phase 完成后确保可用
3. **性能监控**: 重构过程中持续关注内存/CPU 使用
4. **用户反馈**: 可发布 alpha 版本收集反馈
