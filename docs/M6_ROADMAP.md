# M6 Milestone - Niri + Workshop 集成

**版本**: v0.5.0  
**主题**: Niri 生态系统集成 + Steam Workshop 支持  
**预估工期**: 3-4 周  
**优先级**: High (战略性里程碑)

---

## 🎯 战略目标

### 核心使命
为 **Niri 滚动平铺混成器**和即将到来的 **Noctalia Shell** 桌面环境提供一流的视频壁纸支持，同时实现 **Steam Workshop 无缝集成**，成为 Niri 生态的标准壁纸解决方案。

### 目标用户
1. **Niri + Arch Linux 用户** (主要目标)
2. Wallpaper Engine 用户迁移到 Wayland
3. Noctalia Shell 早期采用者
4. 追求简洁高效桌面环境的 Linux 用户

### 成功指标
- 在 Niri 上零配置运行
- Workshop 壁纸一键导入
- Noctalia Shell 无缝集成
- AUR 包周下载量 > 100
- 社区正面反馈 > 90%

---

## 📋 里程碑结构

### Phase 0: 核心集成 (Week 1) - Critical
**目标**: 建立 Workshop 和 Niri 的基础设施

- **Issue #23**: Steam Workshop Integration
  - Steam 库发现
  - Workshop 项目扫描
  - 元数据解析
  - CLI 命令 (`wayvid workshop list/info/import`)
  - **工时**: 12h
  - **优先级**: P0

- **Issue #24**: Niri-Specific Optimizations
  - Niri 兼容性全面测试
  - 工作区感知优化
  - 滚动场景性能调优
  - Niri 配置集成
  - **工时**: 14h
  - **优先级**: P0

**Week 1 目标**: Workshop 基础可用 + Niri 完美运行

---

### Phase 1: 用户体验 (Week 2) - High
**目标**: 提升易用性，降低使用门槛

- **Issue #25**: Arch Linux Packaging Improvements
  - AUR 包优化
  - Niri 配置包 (wayvid-niri-config)
  - Systemd 集成改进
  - 一键安装脚本
  - **工时**: 10h
  - **优先级**: P1

- **Issue #3**: Playlist Support (M5 遗留)
  - 目录视频源
  - 轮播间隔
  - 淡入淡出过渡
  - Workshop 播放列表集成
  - **工时**: 14h
  - **优先级**: P1

- **Issue #6**: Configuration Validator
  - 配置文件验证
  - 错误提示改进
  - 自动修复建议
  - **工时**: 8h
  - **优先级**: P2

**Week 2 目标**: 新手友好 + 播放列表完成

---

### Phase 2: Noctalia 准备 (Week 3) - Medium
**目标**: 为 Noctalia Shell 做好技术准备

- **新 Issue**: Noctalia Shell Integration Preparation
  - D-Bus 接口设计
  - 壁纸管理 API
  - 主题系统集成
  - 配置 GUI 后端
  - **工时**: 16h
  - **优先级**: P1

- **Issue #7**: Interactive Setup Wizard
  - 首次运行向导
  - 自动配置生成
  - Workshop 浏览器集成
  - 硬件检测和优化建议
  - **工时**: 10h
  - **优先级**: P2

- **Issue #8**: Diagnostic Tools
  - 性能监控面板
  - 问题自动诊断
  - 日志分析工具
  - **工时**: 8h
  - **优先级**: P2

**Week 3 目标**: Noctalia 技术栈就绪 + 诊断工具完善

---

### Phase 3: 生态完善 (Week 4) - Low
**目标**: 文档、测试和社区建设

- **文档工作**
  - 创建 `docs/NIRI_INTEGRATION.md`
  - 创建 `docs/WORKSHOP_GUIDE.md`
  - 创建 `docs/NOCTALIA_ROADMAP.md`
  - 更新 README 和 QUICKSTART
  - Arch Wiki 页面草稿
  - **工时**: 12h

- **测试和质量保证**
  - Niri 多场景测试套件
  - Workshop 集成测试
  - 性能回归测试
  - 24 小时稳定性测试
  - **工时**: 10h

- **社区建设**
  - Niri Discord 宣传
  - Reddit r/unixporn 展示
  - AUR 包推广
  - 创建演示视频
  - **工时**: 8h

**Week 4 目标**: 文档完善 + 社区认可

---

## 🏗️ 技术架构

### Workshop 集成架构

```
src/
├── we/
│   ├── mod.rs
│   ├── parser.rs          # WE 项目解析器
│   ├── converter.rs       # WE → wayvid 转换器
│   ├── types.rs           # WE 数据类型
│   ├── steam.rs           # 新增: Steam 库管理
│   │   ├── SteamLibrary
│   │   ├── find_steam_root()
│   │   └── parse_libraryfolders_vdf()
│   └── workshop.rs        # 新增: Workshop API
│       ├── WorkshopItem
│       ├── WorkshopScanner
│       └── WorkshopCache
```

### Niri 优化架构

```
src/
├── backend/
│   └── wayland/
│       ├── mod.rs
│       ├── app.rs
│       ├── output.rs
│       ├── surface.rs
│       └── niri.rs        # 新增: Niri 特定逻辑
│           ├── NiriWorkspaceMonitor
│           ├── NiriScrollDetector
│           └── NiriPowerManager
```

### Noctalia 集成架构

```
src/
├── dbus/                  # 新增: D-Bus 接口
│   ├── mod.rs
│   ├── wallpaper_manager.rs
│   └── org.wayvid.Manager.xml
└── noctalia/              # 新增: Noctalia 适配
    ├── mod.rs
    ├── theme_bridge.rs
    └── config_sync.rs
```

---

## 🔧 关键技术方案

### 1. Workshop 自动发现

```rust
// src/we/steam.rs
pub struct SteamLibrary {
    root_path: PathBuf,
    library_folders: Vec<PathBuf>,
}

impl SteamLibrary {
    pub fn discover() -> Result<Self> {
        // 1. 检查 ~/.steam/steam
        // 2. 检查 ~/.local/share/Steam
        // 3. 解析 libraryfolders.vdf
        // 4. 返回所有库路径
    }
    
    pub fn find_workshop_items(&self, app_id: u32) -> Result<Vec<WorkshopItem>> {
        // 扫描 steamapps/workshop/content/{app_id}/
        // 解析每个项目的 project.json
        // 返回元数据列表
    }
}

pub struct WorkshopItem {
    pub id: u64,
    pub title: String,
    pub path: PathBuf,
    pub preview: Option<PathBuf>,
    pub metadata: WeProject,
}
```

### 2. Niri 工作区感知

```rust
// src/backend/wayland/niri.rs
pub struct NiriWorkspaceMonitor {
    active_workspace: u32,
    visible_outputs: HashSet<String>,
}

impl NiriWorkspaceMonitor {
    pub fn on_workspace_change(&mut self, new_ws: u32) {
        if self.active_workspace != new_ws {
            // 暂停旧工作区播放
            // 恢复新工作区播放
        }
    }
    
    pub fn on_scroll_start(&mut self) {
        // 降低帧率或质量
    }
    
    pub fn on_scroll_end(&mut self) {
        // 恢复正常质量
    }
}
```

### 3. Noctalia D-Bus 接口

```rust
// src/dbus/wallpaper_manager.rs
#[dbus_interface(name = "org.wayvid.WallpaperManager")]
impl WallpaperManager {
    async fn list_wallpapers(&self) -> Vec<WallpaperInfo>;
    async fn set_wallpaper(&self, id: &str, output: &str) -> Result<()>;
    async fn get_current_wallpaper(&self, output: &str) -> Option<String>;
    async fn import_workshop_item(&self, workshop_id: u64) -> Result<String>;
}
```

---

## 📦 新增依赖

```toml
[dependencies]
# VDF 解析 (Steam 配置文件)
keyvalues-parser = "0.2"

# 跨平台目录
dirs = "5.0"

# D-Bus 支持 (Noctalia 集成)
zbus = { version = "4.0", optional = true }

# 特性标志
[features]
workshop = ["dep:keyvalues-parser", "dep:dirs"]
dbus = ["dep:zbus"]
noctalia = ["dbus"]
```

---

## 🧪 测试策略

### Workshop 测试
- [x] Steam 未安装
- [x] Steam 默认路径
- [x] Steam 自定义路径
- [x] 多个 Steam 库
- [x] 0/1/10/100+ Workshop 项目
- [x] 损坏的 project.json

### Niri 测试
- [x] 单工作区单显示器
- [x] 多工作区单显示器
- [x] 多工作区多显示器
- [x] 工作区快速切换
- [x] 输出热插拔
- [x] 24 小时稳定性

### 集成测试
- [x] Workshop → Niri 端到端流程
- [x] 配置热重载
- [x] 性能基准测试
- [x] 内存泄漏检测

---

## 📊 工时预算

| 阶段 | 工时 | 优先级 |
|------|------|--------|
| **Phase 0: 核心集成** | 26h | P0 |
| - Workshop Integration (#23) | 12h | P0 |
| - Niri Optimizations (#24) | 14h | P0 |
| **Phase 1: 用户体验** | 32h | P1 |
| - Arch Packaging (#25) | 10h | P1 |
| - Playlist Support (#3) | 14h | P1 |
| - Config Validator (#6) | 8h | P2 |
| **Phase 2: Noctalia 准备** | 34h | P1-P2 |
| - Noctalia Integration | 16h | P1 |
| - Setup Wizard (#7) | 10h | P2 |
| - Diagnostic Tools (#8) | 8h | P2 |
| **Phase 3: 生态完善** | 30h | P2 |
| - 文档工作 | 12h | P2 |
| - 测试和 QA | 10h | P2 |
| - 社区建设 | 8h | P2 |
| **总计** | **122h** | **~3-4 周** |

---

## 🎯 发布标准

### v0.5.0-alpha (Week 2)
- ✅ Workshop 基础功能可用
- ✅ Niri 兼容性验证
- ✅ 播放列表支持

### v0.5.0-beta (Week 3)
- ✅ AUR 包更新
- ✅ Noctalia 接口设计完成
- ✅ 设置向导可用

### v0.5.0 (Week 4)
- ✅ 所有 P0/P1 功能完成
- ✅ 文档完整
- ✅ 测试覆盖率 > 70%
- ✅ 社区反馈积极

---

## 🌟 创新点

1. **首个 Niri 原生壁纸引擎**
   - 工作区感知
   - 滚动优化
   - 完美集成

2. **Steam Workshop 无缝集成**
   - 一键导入
   - 自动发现
   - 元数据保留

3. **为 Noctalia Shell 铺路**
   - D-Bus 接口
   - 主题集成
   - GUI 就绪

4. **Arch Linux 最佳实践**
   - AUR 包优化
   - Systemd 集成
   - 文档完善

---

## 🚀 后续里程碑预览

### M7: GUI 和完整 Noctalia 集成
- GTK4/Libadwaita 设置 GUI
- Noctalia 控制面板插件
- 主题同步
- 视觉壁纸编辑器

### M8: 高级特性
- 音频反应性 (#4)
- 交互式壁纸
- Shader 系统
- 插件架构

---

## 📞 社区参与

### 贡献机会
- Niri 场景测试
- Workshop 项目测试
- 文档翻译
- 配置模板贡献
- Bug 报告和反馈

### 沟通渠道
- GitHub Issues: 技术问题
- GitHub Discussions: 功能讨论
- Niri Discord: 社区交流
- Reddit: 展示和反馈

---

## 📝 Checklist

### Phase 0
- [ ] Issue #23 完成 (Workshop)
- [ ] Issue #24 完成 (Niri)
- [ ] 基础集成测试通过

### Phase 1
- [ ] Issue #25 完成 (Arch)
- [ ] Issue #3 完成 (Playlist)
- [ ] Issue #6 完成 (Validator)
- [ ] AUR 包更新

### Phase 2
- [ ] Noctalia 接口设计
- [ ] Issue #7 完成 (Wizard)
- [ ] Issue #8 完成 (Diagnostic)

### Phase 3
- [ ] 文档完成
- [ ] 测试通过
- [ ] 社区反馈收集

### 发布
- [ ] 版本号更新到 0.5.0
- [ ] CHANGELOG 更新
- [ ] Release Notes 撰写
- [ ] 标签和发布
- [ ] 社区公告

---

**Document Version**: 1.0  
**Created**: 2025-11-10  
**Status**: 📋 **PLANNING**  
**Next**: 开始 Phase 0 实现
