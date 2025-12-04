# v0.5 GUI-First 重构 - 任务清单

## Phase 1: 项目结构重组 (Week 1)

### 1.1 Workspace 初始化
- [x] 创建 workspace Cargo.toml
- [x] 迁移 src/core → crates/wayvid-core
- [ ] 迁移 src/video + src/backend → crates/wayvid-engine
- [x] 创建 crates/wayvid-library (新模块)
- [ ] 迁移 src/bin/wayvid-gui.rs → crates/wayvid/

### 1.2 依赖整理
- [x] 整理各 crate 依赖关系
- [ ] 移除未使用的依赖
- [x] 统一版本号管理

### 1.3 构建验证
- [x] 确保所有 crate 独立编译
- [ ] 更新 CI 配置
- [ ] 验证现有功能不受影响

## Phase 2: 壁纸库模块 (Week 2)

### 2.1 数据库设计
- [ ] 设计 SQLite schema (wallpapers, folders, thumbnails)
- [ ] 实现 database.rs CRUD 操作
- [ ] 添加增量更新逻辑

### 2.2 扫描器重构
- [ ] 将扫描逻辑从 GUI 移到 wayvid-library
- [ ] 实现异步扫描 (tokio/rayon)
- [ ] 支持增量扫描 (只扫描变更)

### 2.3 缩略图系统
- [ ] 实现缩略图生成 (ffmpeg 或 image crate)
- [ ] WebP 输出格式
- [ ] 缓存目录结构 (~/.cache/wayvid/thumbnails/)
- [ ] GIF 首帧提取
- [ ] 后台线程生成

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
