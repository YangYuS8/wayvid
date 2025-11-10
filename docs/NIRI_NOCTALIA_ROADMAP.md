# Niri + Noctalia Shell 集成路线图

**目标**: 为 Niri 桌面环境生态做出贡献，提供完整的动态壁纸解决方案  
**平台重点**: Arch Linux + Niri + Noctalia Shell  
**特色功能**: Steam Workshop 创意工坊自动集成

---

## 🎯 核心目标

### 1. Niri 优先优化
- ✅ 已支持 Niri (wlr-layer-shell)
- 🔜 Niri 特定优化和测试
- 🔜 Noctalia Shell 深度集成
- 🔜 Niri 社区推广

### 2. Steam Workshop 自动集成
- ✅ 已支持手动导入 WE 项目
- 🔜 自动扫描 Steam Workshop 订阅
- 🔜 壁纸库管理界面
- 🔜 一键应用创意工坊壁纸

### 3. Arch Linux 生态深度整合
- ✅ AUR 包 (wayvid-git)
- 🔜 官方仓库候选
- 🔜 Arch Wiki 文档
- 🔜 与其他 Arch/Niri 工具协作

---

## 📋 开发阶段

### Phase 1: Niri 深度优化 (Week 1-2)

#### 1.1 Niri 特定测试和文档
**优先级**: P0 - 关键

**任务**:
- [ ] 在真实 Niri 环境中全面测试所有功能
- [ ] 测试多显示器配置（横向、纵向、混合）
- [ ] 测试 Niri 特有功能（滚动平铺、工作区切换）
- [ ] 验证与 Niri 窗口管理的兼容性
- [ ] 性能基准测试（CPU、GPU、内存）
- [ ] 创建 Niri 专用配置示例
- [ ] 编写 Niri 集成指南文档

**成功标准**:
- ✅ 所有功能在 Niri 上稳定运行
- ✅ 性能达标（<5% CPU，<200MB 内存）
- ✅ 与 Niri 工作区切换无冲突
- ✅ 完整的 Niri 用户文档

**预估工时**: 8-10 小时

#### 1.2 Noctalia Shell 集成准备
**优先级**: P1 - 高

**任务**:
- [ ] 研究 Noctalia Shell 架构和 API
- [ ] 确定集成点（设置面板、通知、快捷键）
- [ ] 设计 D-Bus 接口供 Noctalia 调用
- [ ] 创建配置预设适配 Noctalia 主题
- [ ] 实现 Noctalia 通知支持
- [ ] 添加 Noctalia 快速设置面板支持

**成功标准**:
- ✅ D-Bus 接口完善
- ✅ 可从 Noctalia 设置调用 wayvid
- ✅ 壁纸切换通知显示
- ✅ 主题联动（可选）

**预估工时**: 10-12 小时

**依赖**: Noctalia Shell API 文档/示例

---

### Phase 2: Steam Workshop 自动集成 (Week 2-3)

#### 2.1 Workshop 订阅扫描器
**优先级**: P0 - 关键

**任务**:
- [ ] 实现 Steam Workshop 目录扫描
  - Linux Steam 路径: `~/.steam/steam/steamapps/workshop/content/431960/`
  - Proton 路径支持
  - 自定义 Steam 库路径检测
- [ ] 解析订阅的壁纸项目
  - 读取 `project.json` 元数据
  - 提取预览图（`preview.jpg`）
  - 分类视频壁纸 vs 其他类型
- [ ] 构建本地壁纸库数据库
  - SQLite 或 JSON 数据库
  - 缓存壁纸元数据
  - 支持快速搜索和过滤
- [ ] 实现自动更新检测
  - 监听 Workshop 目录变化
  - 自动刷新新订阅的壁纸

**数据结构**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopItem {
    pub workshop_id: String,
    pub title: String,
    pub description: String,
    pub preview_path: Option<PathBuf>,
    pub video_path: PathBuf,
    pub project_type: String,
    pub tags: Vec<String>,
    pub properties: WeProperties,
    pub subscribed_date: Option<SystemTime>,
    pub last_updated: Option<SystemTime>,
}

#[derive(Debug)]
pub struct WorkshopLibrary {
    items: HashMap<String, WorkshopItem>,
    steam_path: PathBuf,
}
```

**CLI 命令**:
```bash
# 扫描并列出所有 Workshop 壁纸
wayvid workshop list

# 按标签过滤
wayvid workshop list --tag Nature

# 搜索壁纸
wayvid workshop search "ocean"

# 显示壁纸详情
wayvid workshop show 2934567890

# 应用壁纸到所有显示器
wayvid workshop apply 2934567890

# 应用到特定显示器
wayvid workshop apply 2934567890 --output HDMI-A-1

# 刷新库
wayvid workshop refresh
```

**成功标准**:
- ✅ 自动检测 Steam 安装路径
- ✅ 扫描所有订阅的视频壁纸
- ✅ 提取完整元数据
- ✅ 快速列表和搜索
- ✅ 一键应用壁纸

**预估工时**: 14-16 小时

#### 2.2 壁纸库管理 UI (CLI)
**优先级**: P1 - 高

**任务**:
- [ ] 实现交互式壁纸选择器
  - 使用 `crossterm` 或 `ratatui` 构建 TUI
  - 显示预览图（如果终端支持）
  - 键盘导航和搜索
- [ ] 实现壁纸收藏系统
  - 标记常用壁纸
  - 创建自定义播放列表
- [ ] 实现壁纸预设
  - 保存多显示器配置方案
  - 快速切换场景（工作、娱乐、夜间）
- [ ] 添加壁纸统计
  - 使用次数
  - 播放时长
  - 评分系统

**TUI 示例**:
```
╔════════════════════════════════════════════════════════════════════════╗
║ WayVid Workshop Library (127 items)                          [F1:Help] ║
╠════════════════════════════════════════════════════════════════════════╣
║ Search: ocean                                      [Tags: Nature (12)] ║
╠════════════════════════════════════════════════════════════════════════╣
║                                                                        ║
║ ┌─────────────┬──────────────────────────────────────────────────┐   ║
║ │ [PREVIEW]   │ Ocean Waves 4K                                   │   ║
║ │             │ Workshop ID: 2934567890                          │   ║
║ │  [IMAGE]    │ Tags: Nature, Ocean, Relaxing                    │   ║
║ │             │ ⭐⭐⭐⭐⭐ (852 ratings)                               │   ║
║ │             │ Used: 15 times • Last: 2 days ago               │   ║
║ └─────────────┴──────────────────────────────────────────────────┘   ║
║                                                                        ║
║ [Enter] Apply  [Space] Favorite  [/] Search  [t] Filter Tags  [q] Quit║
╚════════════════════════════════════════════════════════════════════════╝
```

**CLI 命令**:
```bash
# 交互式选择器
wayvid workshop select

# 收藏管理
wayvid workshop favorite add 2934567890
wayvid workshop favorite list
wayvid workshop favorite apply  # 应用收藏夹中的随机壁纸

# 预设管理
wayvid workshop preset save work  # 保存当前配置为 "work"
wayvid workshop preset apply work
wayvid workshop preset list
```

**成功标准**:
- ✅ 流畅的 TUI 体验
- ✅ 预览图显示（支持的终端）
- ✅ 收藏和预设系统完善
- ✅ 与 `wayvid-ctl` 命令协同工作

**预估工时**: 12-14 小时

---

### Phase 3: 播放列表与随机播放 (Week 3)

#### 3.1 Workshop 播放列表集成
**优先级**: P1 - 高

**任务**:
- [ ] 基于 Issue #3 实现播放列表功能
- [ ] 扩展支持 Workshop 集合
  - 从标签创建播放列表
  - 从收藏夹创建播放列表
  - 从搜索结果创建播放列表
- [ ] 实现智能轮播
  - 时间间隔轮播
  - 随机播放模式
  - 基于时间段自动切换（白天/夜晚）
- [ ] 添加过渡效果
  - 淡入淡出
  - 滑动过渡
  - 自定义过渡时长

**配置示例**:
```yaml
# 使用 Workshop 标签创建播放列表
source:
  type: WorkshopPlaylist
  tags: ["Nature", "Relaxing"]
  shuffle: true
  interval: 600  # 10 分钟轮播
  transition: crossfade
  transition_duration: 2.0  # 2 秒过渡

# 使用收藏夹
source:
  type: WorkshopPlaylist
  favorites: true
  shuffle: true
  interval: 300

# 时间段切换
source:
  type: WorkshopPlaylist
  schedule:
    - time: "06:00-18:00"
      tags: ["Bright", "Energetic"]
    - time: "18:00-06:00"
      tags: ["Dark", "Calm"]
```

**CLI 命令**:
```bash
# 创建并应用播放列表
wayvid workshop playlist create nature --tags Nature,Ocean,Forest
wayvid workshop playlist apply nature --shuffle --interval 600

# 列出播放列表
wayvid workshop playlist list

# 手动切换到下一个
wayvid-ctl next
wayvid-ctl prev
```

**成功标准**:
- ✅ 流畅的壁纸切换
- ✅ 支持多种播放模式
- ✅ 过渡效果自然
- ✅ 低资源占用

**预估工时**: 10-12 小时

---

### Phase 4: Noctalia Shell 深度集成 (Week 4)

#### 4.1 GUI 设置面板
**优先级**: P1 - 高

**任务**:
- [ ] 创建 Noctalia 设置插件
  - GTK4/Adwaita 界面
  - 集成到 Noctalia 设置中心
- [ ] 实现可视化壁纸选择器
  - 网格视图显示预览图
  - 缩略图懒加载
  - 实时预览
- [ ] 添加快速设置
  - 暂停/恢复壁纸
  - 快速切换预设
  - 性能模式切换（省电/性能/平衡）
- [ ] 添加配置编辑器
  - 布局模式选择（可视化）
  - 音量和播放速度调整
  - 显示器配置管理

**界面设计草图**:
```
┌─────────────────────────────────────────────────────────────┐
│ ⚙ WayVid 壁纸设置                                    [✕]    │
├─────────────────────────────────────────────────────────────┤
│ 📚 Workshop 库 │ 🎨 收藏 │ ⚡ 预设 │ ⚙️ 设置              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐        │
│ │[图片] │ │[图片] │ │[图片] │ │[图片] │ │[图片] │        │
│ │Ocean  │ │Forest │ │Space  │ │City   │ │Abstract│       │
│ │⭐⭐⭐⭐│ │⭐⭐⭐⭐⭐│ │⭐⭐⭐  │ │⭐⭐⭐⭐│ │⭐⭐⭐⭐⭐│       │
│ └───────┘ └───────┘ └───────┘ └───────┘ └───────┘        │
│                                                             │
│ 当前播放: Ocean Waves 4K                                    │
│ [▶ 播放中] [⏸ 暂停] [⏭ 下一个] [🔀 随机]                  │
│                                                             │
│ 显示器配置:                                                 │
│ ├─ HDMI-A-1 (主显示器) ────────────── ✅ 启用              │
│ └─ DP-1 (副显示器) ────────────────── ✅ 启用              │
│                                                             │
│ 布局: ◉ Fill  ○ Contain  ○ Cover  ○ Centre                │
│ 音量: ▬▬▬▬▬▬▬▬▬▬░░░░░░░ 65%                                │
│ 速度: ▬▬▬▬▬░░░░░░░░░░░░ 1.0x                               │
│                                                             │
│ 性能模式: ◉ 平衡  ○ 性能  ○ 省电                           │
│                                                             │
│                      [应用] [取消]                          │
└─────────────────────────────────────────────────────────────┘
```

**D-Bus 接口**:
```xml
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
"http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node>
  <interface name="io.github.wayvid.Manager">
    <!-- Methods -->
    <method name="GetWorkshopLibrary">
      <arg direction="out" type="a(sssas)" name="items"/>
    </method>
    <method name="ApplyWallpaper">
      <arg direction="in" type="s" name="workshop_id"/>
      <arg direction="in" type="s" name="output"/>
    </method>
    <method name="GetCurrentWallpaper">
      <arg direction="in" type="s" name="output"/>
      <arg direction="out" type="(ss)" name="wallpaper"/>
    </method>
    <method name="PausePlayback">
      <arg direction="in" type="s" name="output"/>
    </method>
    <method name="ResumePlayback">
      <arg direction="in" type="s" name="output"/>
    </method>
    <method name="NextWallpaper">
      <arg direction="in" type="s" name="output"/>
    </method>
    
    <!-- Signals -->
    <signal name="WallpaperChanged">
      <arg type="s" name="output"/>
      <arg type="s" name="workshop_id"/>
    </signal>
    <signal name="LibraryUpdated">
      <arg type="i" name="item_count"/>
    </signal>
  </interface>
</node>
```

**实现**:
```rust
// src/dbus/manager.rs
use zbus::{dbus_interface, Connection};

pub struct WayvidManager {
    workshop: Arc<Mutex<WorkshopLibrary>>,
    app: Arc<Mutex<WaylandApp>>,
}

#[dbus_interface(name = "io.github.wayvid.Manager")]
impl WayvidManager {
    async fn get_workshop_library(&self) -> Vec<(String, String, String, Vec<String>)> {
        // 返回 (id, title, preview_path, tags)
        self.workshop.lock().await.get_all_items()
    }

    async fn apply_wallpaper(&self, workshop_id: String, output: String) -> zbus::fdo::Result<()> {
        // 应用指定壁纸
        self.app.lock().await.apply_workshop_wallpaper(&workshop_id, &output)?;
        
        // 发送信号
        self.wallpaper_changed(output.clone(), workshop_id.clone()).await?;
        Ok(())
    }

    #[dbus_interface(signal)]
    async fn wallpaper_changed(
        &self,
        signal_ctxt: &zbus::SignalContext<'_>,
        output: String,
        workshop_id: String,
    ) -> zbus::Result<()>;
}
```

**成功标准**:
- ✅ GUI 集成到 Noctalia 设置
- ✅ 可视化壁纸选择流畅
- ✅ D-Bus 接口完善
- ✅ 实时预览功能正常
- ✅ 符合 Noctalia 设计规范

**预估工时**: 16-20 小时

**依赖**:
- GTK4 / libadwaita
- zbus (D-Bus)
- Noctalia Shell API 文档

---

### Phase 5: 社区推广与文档 (Week 4-5)

#### 5.1 Arch Linux 社区推广
**优先级**: P2 - 中

**任务**:
- [ ] 提升 AUR 包质量
  - 添加更多可选依赖说明
  - 优化构建脚本
  - 添加 `.install` 脚本（提示配置）
- [ ] 提交到 [community] 仓库
  - 满足 TU 要求
  - 准备投票材料
- [ ] 编写 Arch Wiki 页面
  - 安装指南
  - 配置示例
  - 故障排除
  - Niri 集成指南
- [ ] 在 r/archlinux 和 Arch 论坛推广
  - 发布介绍帖
  - 回答用户问题
  - 收集反馈

**成功标准**:
- ✅ AUR 包投票数 >50
- ✅ Arch Wiki 页面完成
- ✅ 社区反馈积极
- 🎯 [community] 仓库候选（长期目标）

**预估工时**: 6-8 小时

#### 5.2 Niri 生态集成
**优先级**: P1 - 高

**任务**:
- [ ] 与 Niri 开发者沟通
  - 提交功能需求（如需要）
  - 协助测试新版本兼容性
  - 贡献代码（如有机会）
- [ ] 集成到 Niri 推荐工具列表
  - 提交 PR 到 Niri 文档
  - 添加到 awesome-niri 列表
- [ ] 与其他 Niri 工具协作
  - 与通知守护进程集成
  - 与状态栏工具协作
  - 与锁屏工具协作（壁纸同步）
- [ ] 创建 Niri 配置模板包
  - 开箱即用的配置
  - 键盘快捷键预设
  - 脚本集成示例

**成功标准**:
- ✅ 被 Niri 官方推荐
- ✅ 与 Noctalia Shell 深度集成
- ✅ 与其他 Niri 工具良好协作

**预估工时**: 8-10 小时

#### 5.3 完善文档和示例
**优先级**: P1 - 高

**任务**:
- [ ] 创建完整的 Niri 用户指南
  - 从零开始安装配置
  - Workshop 集成教程
  - 多显示器配置示例
  - 性能优化建议
  - 故障排除指南
- [ ] 录制演示视频
  - 安装过程
  - 基本使用
  - Workshop 功能展示
  - 高级配置
- [ ] 创建配置示例库
  - 单显示器配置
  - 双显示器配置
  - 三显示器+配置
  - 笔记本+外接显示器
  - 垂直显示器配置
- [ ] 翻译文档
  - 中文文档（简体/繁体）
  - 英文文档（已有）
  - 其他语言（社区贡献）

**文档结构**:
```
docs/
├── niri/
│   ├── README.md                  # Niri 集成总览
│   ├── INSTALLATION.md            # 详细安装指南
│   ├── CONFIGURATION.md           # 配置说明
│   ├── WORKSHOP_GUIDE.md          # Workshop 使用教程
│   ├── NOCTALIA_INTEGRATION.md    # Noctalia 集成文档
│   ├── MULTI_MONITOR.md           # 多显示器配置
│   ├── PERFORMANCE.md             # 性能优化
│   └── TROUBLESHOOTING.md         # 故障排除
├── examples/
│   ├── niri/
│   │   ├── single-monitor.yaml
│   │   ├── dual-monitor.yaml
│   │   ├── triple-monitor.yaml
│   │   ├── laptop-external.yaml
│   │   ├── vertical-monitor.yaml
│   │   └── workshop-playlist.yaml
│   └── noctalia/
│       ├── integration-example.yaml
│       └── quick-settings.yaml
└── zh-CN/                         # 中文文档
    └── niri/
        └── ...
```

**成功标准**:
- ✅ 完整的 Niri 用户文档
- ✅ 至少 2 个演示视频
- ✅ 10+ 配置示例
- ✅ 中英双语文档

**预估工时**: 10-12 小时

---

## 🛠️ 技术实现细节

### Workshop 扫描器实现

```rust
// src/workshop/scanner.rs
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub struct WorkshopScanner {
    steam_path: PathBuf,
    workshop_path: PathBuf,
}

impl WorkshopScanner {
    /// 自动检测 Steam 安装路径
    pub fn detect_steam_path() -> Result<PathBuf> {
        let possible_paths = vec![
            PathBuf::from(shellexpand::tilde("~/.steam/steam")),
            PathBuf::from(shellexpand::tilde("~/.local/share/Steam")),
            PathBuf::from("/usr/share/steam"),
        ];

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!("Steam installation not found"))
    }

    /// 创建扫描器
    pub fn new() -> Result<Self> {
        let steam_path = Self::detect_steam_path()?;
        let workshop_path = steam_path
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join("431960"); // Wallpaper Engine App ID

        Ok(Self {
            steam_path,
            workshop_path,
        })
    }

    /// 扫描所有订阅的壁纸
    pub fn scan_all(&self) -> Result<Vec<WorkshopItem>> {
        let mut items = Vec::new();

        if !self.workshop_path.exists() {
            return Err(anyhow::anyhow!(
                "Workshop directory not found: {}",
                self.workshop_path.display()
            ));
        }

        for entry in fs::read_dir(&self.workshop_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Ok(item) = self.scan_item(&path) {
                    items.push(item);
                }
            }
        }

        Ok(items)
    }

    /// 扫描单个壁纸项目
    fn scan_item(&self, path: &Path) -> Result<WorkshopItem> {
        let workshop_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid workshop ID"))?
            .to_string();

        // 解析 project.json
        let project_file = path.join("project.json");
        if !project_file.exists() {
            return Err(anyhow::anyhow!("No project.json found"));
        }

        let (project, video_path) = crate::we::parse_we_project(&project_file)?;

        // 检查是否为视频壁纸
        if project.project_type != "video" {
            return Err(anyhow::anyhow!(
                "Not a video wallpaper: {}",
                project.project_type
            ));
        }

        // 提取预览图路径
        let preview_path = project.preview.as_ref().map(|p| path.join(p));

        // 获取文件元数据
        let metadata = fs::metadata(&video_path).ok();
        let subscribed_date = metadata.as_ref().and_then(|m| m.created().ok());
        let last_updated = metadata.as_ref().and_then(|m| m.modified().ok());

        Ok(WorkshopItem {
            workshop_id,
            title: project.title,
            description: project.description,
            preview_path,
            video_path,
            project_type: project.project_type,
            tags: project.tags,
            properties: crate::we::extract_properties(&project),
            subscribed_date,
            last_updated,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopItem {
    pub workshop_id: String,
    pub title: String,
    pub description: String,
    pub preview_path: Option<PathBuf>,
    pub video_path: PathBuf,
    pub project_type: String,
    pub tags: Vec<String>,
    pub properties: crate::we::types::WeProperties,
    pub subscribed_date: Option<std::time::SystemTime>,
    pub last_updated: Option<std::time::SystemTime>,
}

impl WorkshopItem {
    /// 转换为 wayvid 配置
    pub fn to_config(&self) -> Result<crate::config::Config> {
        crate::we::converter::generate_wayvid_config(&self.properties, self.video_path.clone())
    }
}
```

### Workshop 库管理器

```rust
// src/workshop/library.rs
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkshopLibrary {
    items: HashMap<String, WorkshopItem>,
    favorites: Vec<String>,
    playlists: HashMap<String, Vec<String>>,
    #[serde(skip)]
    cache_path: PathBuf,
}

impl WorkshopLibrary {
    /// 加载库（从缓存或重新扫描）
    pub fn load() -> Result<Self> {
        let cache_path = Self::cache_path()?;

        if cache_path.exists() {
            let content = fs::read_to_string(&cache_path)?;
            let mut library: Self = serde_json::from_str(&content)?;
            library.cache_path = cache_path;
            Ok(library)
        } else {
            Self::scan_and_save()
        }
    }

    /// 扫描并保存
    pub fn scan_and_save() -> Result<Self> {
        let scanner = WorkshopScanner::new()?;
        let items = scanner.scan_all()?;

        let mut library = Self {
            items: items
                .into_iter()
                .map(|item| (item.workshop_id.clone(), item))
                .collect(),
            favorites: Vec::new(),
            playlists: HashMap::new(),
            cache_path: Self::cache_path()?,
        };

        library.save()?;
        Ok(library)
    }

    /// 保存到缓存
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&self.cache_path, content)?;
        Ok(())
    }

    /// 缓存路径
    fn cache_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?;
        let cache_dir = config_dir.join("wayvid");
        fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir.join("workshop_library.json"))
    }

    /// 搜索壁纸
    pub fn search(&self, query: &str) -> Vec<&WorkshopItem> {
        let query = query.to_lowercase();
        self.items
            .values()
            .filter(|item| {
                item.title.to_lowercase().contains(&query)
                    || item.description.to_lowercase().contains(&query)
                    || item.tags.iter().any(|t| t.to_lowercase().contains(&query))
            })
            .collect()
    }

    /// 按标签过滤
    pub fn filter_by_tags(&self, tags: &[String]) -> Vec<&WorkshopItem> {
        self.items
            .values()
            .filter(|item| tags.iter().any(|tag| item.tags.contains(tag)))
            .collect()
    }

    /// 添加收藏
    pub fn add_favorite(&mut self, workshop_id: String) -> Result<()> {
        if !self.favorites.contains(&workshop_id) {
            self.favorites.push(workshop_id);
            self.save()?;
        }
        Ok(())
    }

    /// 获取收藏列表
    pub fn get_favorites(&self) -> Vec<&WorkshopItem> {
        self.favorites
            .iter()
            .filter_map(|id| self.items.get(id))
            .collect()
    }

    /// 创建播放列表
    pub fn create_playlist(&mut self, name: String, items: Vec<String>) -> Result<()> {
        self.playlists.insert(name, items);
        self.save()?;
        Ok(())
    }
}
```

---

## 📅 时间线

### Week 1: Niri 深度优化
- Days 1-2: Niri 环境测试和文档
- Days 3-4: Noctalia Shell 研究和设计
- Days 5-7: D-Bus 接口实现

### Week 2: Workshop 扫描器
- Days 1-3: Workshop 扫描和解析
- Days 4-5: 本地库数据库
- Days 6-7: CLI 命令实现

### Week 3: TUI 和播放列表
- Days 1-3: 交互式壁纸选择器
- Days 4-5: 收藏和预设系统
- Days 6-7: 播放列表集成

### Week 4: Noctalia GUI
- Days 1-4: GTK4 设置面板
- Days 5-7: 可视化选择器和预览

### Week 5: 文档和推广
- Days 1-3: 完善文档和示例
- Days 4-5: 录制演示视频
- Days 6-7: 社区推广

---

## 🎯 成功指标

### 技术指标
- ✅ 在 Niri 上稳定运行（零崩溃）
- ✅ CPU 使用率 < 5%（单显示器）
- ✅ 内存使用 < 150MB（单显示器）
- ✅ Workshop 扫描 < 5 秒（100 项）
- ✅ 壁纸切换延迟 < 500ms
- ✅ GUI 响应时间 < 100ms

### 用户体验指标
- ✅ 从安装到使用 < 5 分钟
- ✅ Workshop 壁纸应用 < 3 次点击
- ✅ 配置文档完整度 > 90%
- ✅ 用户满意度 > 4.5/5

### 社区指标
- 🎯 AUR 投票数 > 100
- 🎯 GitHub Stars > 200
- 🎯 被 Niri 官方推荐
- 🎯 Noctalia Shell 默认集成
- 🎯 Arch Wiki 页面创建

---

## 🔧 依赖和工具

### 新增依赖
```toml
[dependencies]
# D-Bus
zbus = "4.0"

# TUI
crossterm = "0.27"
ratatui = "0.25"

# GUI (可选)
gtk4 = { version = "0.8", optional = true }
libadwaita = { version = "0.6", optional = true }

# 目录管理
dirs = "5.0"

# 数据库（如果需要）
rusqlite = { version = "0.31", optional = true }

[features]
default = ["video-mpv", "backend-wayland", "workshop"]
workshop = []  # Workshop 集成
dbus = ["dep:zbus"]
gui = ["dep:gtk4", "dep:libadwaita"]
tui = ["dep:crossterm", "dep:ratatui"]
```

### 开发工具
- GTK4 开发库
- libadwaita
- D-Bus 开发工具
- Niri 最新版本（测试环境）

---

## 📞 社区协作

### Niri 社区
- GitHub: https://github.com/YaLTeR/niri
- Discord/Matrix: （待确认）

### Noctalia Shell
- GitHub: （待确认）
- 联系方式: （待确认）

### Arch Linux
- AUR: https://aur.archlinux.org/packages/wayvid-git
- 论坛: https://bbs.archlinux.org/
- Wiki: https://wiki.archlinux.org/

---

## 📝 附录

### A. Steam Workshop 路径说明

**Linux 默认路径**:
```
~/.steam/steam/steamapps/workshop/content/431960/
```

**目录结构**:
```
431960/
├── 2934567890/          # Workshop ID
│   ├── project.json
│   ├── preview.jpg
│   └── video.mp4
├── 2945678901/
│   └── ...
└── ...
```

### B. Noctalia Shell API 参考

（待 Noctalia Shell 文档完善后补充）

### C. 配置迁移指南

从手动配置迁移到 Workshop 集成：

**旧配置**:
```yaml
source:
  type: File
  path: "~/.steam/steam/steamapps/workshop/content/431960/2934567890/video.mp4"
```

**新配置**:
```yaml
source:
  type: WorkshopItem
  workshop_id: "2934567890"
```

或者直接使用 CLI:
```bash
wayvid workshop apply 2934567890
```

---

**文档版本**: 1.0  
**创建日期**: 2025-11-10  
**状态**: 📋 规划中  
**下一步**: Phase 1 实施
