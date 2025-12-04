# v0.5 GUI-First 重构提案

## 概述

**目标**: 将 wayvid 从 CLI 优先转变为 GUI 优先的桌面应用，对标 Wallpaper Engine 的用户体验，同时保持轻量和高性能。

**核心理念**: 
- 打开 GUI → 选壁纸 → 应用，用户无需记命令
- GUI 内置后台服务，无需单独启动 daemon
- 配置通过 GUI 完成，自动保存
- CLI 保留但极简化，面向脚本集成

## 动机

### 当前问题

1. **CLI 优先设计不符合用户预期**
   - 壁纸软件用户期望 GUI 操作
   - 社区反馈：命令行工具"意义不明"
   - 需要手写 config.yaml 令普通用户困惑

2. **GUI 代码臃肿**
   - wayvid-gui.rs 超过 2100 行单文件
   - 壁纸扫描/图片加载阻塞 UI
   - 难以测试和维护

3. **缩略图处理不友好**
   - 每次启动重新扫描
   - GIF 全帧加载占用大量内存
   - 无缓存机制

4. **前后端耦合**
   - GUI 直接调用底层模块
   - 壁纸库管理逻辑分散

## 设计方案

### 新架构

```
┌─────────────────────────────────────────────────────────────┐
│                     wayvid (主程序)                          │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                    GUI 界面                           │   │
│  │  ┌─────────┐  ┌──────────┐  ┌──────────────────┐    │   │
│  │  │ 壁纸库  │  │ 显示器   │  │      设置        │    │   │
│  │  │         │  │ 预览     │  │ (自启动/性能等)  │    │   │
│  │  └─────────┘  └──────────┘  └──────────────────┘    │   │
│  └──────────────────────────────────────────────────────┘   │
│                            │                                 │
│                            ▼                                 │
│  ┌──────────────────────────────────────────────────────┐   │
│  │               内置后台服务                             │   │
│  │  - Wayland 壁纸渲染                                   │   │
│  │  - 壁纸库索引与缩略图                                  │   │
│  │  - 自动保存配置                                       │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘

可选：wayvid-ctl (精简 CLI，面向脚本)
```

### Workspace 结构

```
wayvid/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── wayvid-core/              # 核心共享类型
│   │   ├── types.rs              # VideoSource, LayoutMode
│   │   ├── config.rs             # AppSettings (自动保存)
│   │   └── library.rs            # WallpaperItem 定义
│   │
│   ├── wayvid-engine/            # 渲染引擎
│   │   ├── wayland.rs            # Wayland layer-shell
│   │   ├── renderer.rs           # 视频/场景渲染
│   │   ├── mpv.rs                # MPV 集成
│   │   └── vulkan/               # Vulkan 后端
│   │
│   ├── wayvid-library/           # 壁纸库管理
│   │   ├── scanner.rs            # 文件夹扫描
│   │   ├── thumbnail.rs          # 缩略图生成与缓存
│   │   ├── database.rs           # SQLite 索引
│   │   └── workshop.rs           # Steam Workshop
│   │
│   └── wayvid/                   # 主程序 (GUI)
│       ├── main.rs               # 入口
│       ├── app.rs                # 应用状态
│       ├── service.rs            # 内置后台服务
│       ├── views/                # UI 视图
│       │   ├── library.rs
│       │   ├── monitors.rs
│       │   └── settings.rs
│       └── widgets/              # UI 组件
│           ├── wallpaper_card.rs
│           ├── thumbnail.rs
│           └── preview.rs
│
└── crates/wayvid-ctl/            # 精简 CLI
    └── main.rs
```

### GUI 设计

**主界面布局**:
- 左侧：文件夹树 + 筛选器
- 中央：壁纸网格 (虚拟滚动)
- 底部：显示器选择栏 + 状态
- 双击应用，右键菜单

**关键 UX**:
- 拖拽添加壁纸
- 异步缩略图加载
- 关闭最小化到托盘
- 开机自启动选项

### 精简 CLI

```bash
wayvid-ctl apply <path>           # 应用壁纸
wayvid-ctl apply <path> -o DP-1   # 指定显示器
wayvid-ctl pause                  # 暂停
wayvid-ctl resume                 # 恢复
wayvid-ctl status                 # JSON 状态
```

移除的复杂命令:
- `wayvid run --config` → GUI 管理
- `wayvid daemon start/stop` → GUI 自动管理
- `wayvid import` → GUI 内完成
- `wayvid workshop ...` → GUI 内完成

### 配置自动化

用户无需手写配置文件:

```yaml
# ~/.config/wayvid/settings.yaml (GUI 自动管理)
autostart: true
minimize_to_tray: true
language: zh-CN

render_backend: auto
fps_limit: 60
pause_on_fullscreen: true
pause_on_battery: true

wallpaper_folders:
  - ~/Videos/Wallpapers
  - ~/.steam/steam/steamapps/workshop/content/431960

active_wallpapers:
  DP-1:
    source: ~/Videos/sunset.mp4
    layout: fill
    volume: 0.0
```

### 缩略图系统

```
~/.cache/wayvid/
├── thumbnails/
│   ├── <hash>.webp        # 缩略图 (WebP 格式)
│   └── <hash>@2x.webp     # HiDPI 缩略图
└── library.db             # SQLite 索引
```

**优化策略**:
- 后台线程生成缩略图
- 只加载可见区域
- GIF 只保存首帧
- WebP 格式节省空间

## 版本规划

- **v0.5.0**: GUI-First 重构完成
- **v0.4.x**: 当前版本，维护但不添加新功能

## 兼容性

- 现有 config.yaml 格式继续支持
- systemd service 保留但标记为可选
- AUR/Nix 打包方式不变

## 风险

1. **工作量大**: 需要 4-6 周完成
2. **破坏性变更**: CLI 命令精简可能影响现有脚本
3. **测试覆盖**: 新架构需要重建测试

## 决策

- [x] 采用 GUI-First 设计
- [x] 废弃复杂 CLI 命令
- [x] 实现缩略图缓存系统
- [x] 采用 Workspace 多 crate 结构
