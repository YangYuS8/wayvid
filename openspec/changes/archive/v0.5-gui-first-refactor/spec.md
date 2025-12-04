# v0.5 GUI-First 重构 - 技术规格

## 1. Crate 依赖关系

```
wayvid-core         ← 基础类型，无外部依赖
    ↑
wayvid-library      ← 依赖 core，添加 SQLite/image
    ↑
wayvid-engine       ← 依赖 core，Wayland/MPV/OpenGL
    ↑
wayvid (GUI)        ← 依赖 core/library/engine，iced (wgpu)
    
wayvid-ctl (CLI)    ← 依赖 core，极简
```

## 2. wayvid-core

### 2.1 VideoSource (保持现有)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoSource {
    File { path: String },
    Directory { path: String },
    Url { url: String },
    WeProject { path: String },
    WeScene { path: String },
}
```

### 2.2 WallpaperItem (新增)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperItem {
    pub id: String,              // SHA256 of path
    pub name: String,
    pub source_path: PathBuf,
    pub source_type: SourceType,
    pub wallpaper_type: WallpaperType,
    pub thumbnail_path: Option<PathBuf>,
    pub metadata: WallpaperMetadata,
    pub added_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    LocalFile,
    LocalDirectory,
    SteamWorkshop { workshop_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WallpaperType {
    Video,
    Scene,
    Gif,
    Image,  // 静态图片支持
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WallpaperMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub duration_secs: Option<f64>,
    pub resolution: Option<(u32, u32)>,
}
```

### 2.3 AppSettings (新增)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // 基本设置
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub language: String,
    pub theme: Theme,
    
    // 性能设置
    pub render_backend: RenderBackend,
    pub fps_limit: Option<u32>,
    pub pause_on_fullscreen: bool,
    pub pause_on_battery: bool,
    
    // 壁纸库
    pub wallpaper_folders: Vec<PathBuf>,
    pub steam_workshop_enabled: bool,
    
    // 活动壁纸
    pub active_wallpapers: HashMap<String, ActiveWallpaper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWallpaper {
    pub source_id: String,      // WallpaperItem.id
    pub source_path: String,    // 备用，防止数据库损坏
    pub layout: LayoutMode,
    pub volume: f32,
    pub playback_rate: f32,
    pub muted: bool,
}

impl AppSettings {
    pub fn load() -> Self { ... }
    pub fn save(&self) -> Result<()> { ... }
    pub fn config_path() -> PathBuf { ... }
}
```

## 3. wayvid-library

### 3.1 SQLite Schema
```sql
-- 壁纸表
CREATE TABLE wallpapers (
    id TEXT PRIMARY KEY,           -- SHA256(source_path)
    name TEXT NOT NULL,
    source_path TEXT NOT NULL UNIQUE,
    source_type TEXT NOT NULL,     -- 'local_file', 'local_dir', 'workshop'
    wallpaper_type TEXT NOT NULL,  -- 'video', 'scene', 'gif', 'image'
    workshop_id INTEGER,
    thumbnail_path TEXT,
    metadata_json TEXT,            -- JSON blob
    file_size INTEGER,
    file_mtime INTEGER,            -- 文件修改时间
    added_at TEXT NOT NULL,
    last_used TEXT
);

-- 文件夹表
CREATE TABLE folders (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    enabled BOOLEAN DEFAULT 1,
    last_scan TEXT
);

-- 缩略图状态
CREATE TABLE thumbnails (
    wallpaper_id TEXT PRIMARY KEY,
    status TEXT NOT NULL,          -- 'pending', 'generating', 'done', 'failed'
    path TEXT,
    generated_at TEXT,
    FOREIGN KEY (wallpaper_id) REFERENCES wallpapers(id)
);

CREATE INDEX idx_wallpapers_type ON wallpapers(wallpaper_type);
CREATE INDEX idx_wallpapers_source ON wallpapers(source_type);
```

### 3.2 Library API
```rust
pub struct WallpaperLibrary {
    db: Connection,
    cache_dir: PathBuf,
    thumbnail_tx: Sender<ThumbnailJob>,
}

impl WallpaperLibrary {
    pub fn open() -> Result<Self>;
    
    // 文件夹管理
    pub fn add_folder(&self, path: &Path) -> Result<()>;
    pub fn remove_folder(&self, path: &Path) -> Result<()>;
    pub fn list_folders(&self) -> Result<Vec<FolderInfo>>;
    
    // 扫描
    pub async fn scan_all(&self) -> Result<ScanResult>;
    pub async fn scan_folder(&self, path: &Path) -> Result<ScanResult>;
    
    // 查询
    pub fn list_wallpapers(&self, filter: &WallpaperFilter) -> Result<Vec<WallpaperItem>>;
    pub fn get_wallpaper(&self, id: &str) -> Result<Option<WallpaperItem>>;
    pub fn search(&self, query: &str) -> Result<Vec<WallpaperItem>>;
    
    // 缩略图
    pub fn get_thumbnail(&self, id: &str) -> Option<PathBuf>;
    pub fn request_thumbnail(&self, id: &str);  // 异步生成
}

#[derive(Default)]
pub struct WallpaperFilter {
    pub wallpaper_type: Option<WallpaperType>,
    pub source_type: Option<SourceType>,
    pub folder: Option<PathBuf>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
```

### 3.3 缩略图生成器
```rust
pub struct ThumbnailGenerator {
    cache_dir: PathBuf,
    rx: Receiver<ThumbnailJob>,
}

impl ThumbnailGenerator {
    /// 后台线程运行
    pub fn run(self) {
        while let Ok(job) = self.rx.recv() {
            match self.generate(&job) {
                Ok(path) => job.callback.send(Ok(path)),
                Err(e) => job.callback.send(Err(e)),
            }
        }
    }
    
    fn generate(&self, job: &ThumbnailJob) -> Result<PathBuf> {
        let output_path = self.cache_dir.join(format!("{}.webp", job.id));
        
        match job.source_type {
            WallpaperType::Video => self.generate_video_thumbnail(&job.path, &output_path),
            WallpaperType::Gif => self.extract_gif_first_frame(&job.path, &output_path),
            WallpaperType::Scene => self.generate_scene_preview(&job.path, &output_path),
            WallpaperType::Image => self.resize_image(&job.path, &output_path),
        }
    }
}
```

## 4. wayvid (GUI)

### 4.1 App 状态 (iced Elm 架构)
```rust
/// Application state
pub struct WayvidApp {
    // 核心服务
    library: Arc<WallpaperLibrary>,
    engine: Option<WallpaperEngine>,
    settings: AppSettings,
    
    // UI 状态
    current_view: View,
    selected_monitor: Option<String>,
    selected_wallpaper: Option<String>,
    search_query: String,
    
    // 缩略图缓存 (iced handles)
    thumbnails: HashMap<String, ThumbnailState>,
    
    // 壁纸列表
    wallpapers: Vec<WallpaperItem>,
}

/// iced Message 类型
#[derive(Debug, Clone)]
pub enum Message {
    // 导航
    SwitchView(View),
    
    // 壁纸库
    SearchChanged(String),
    WallpaperSelected(String),
    WallpaperApply(String),
    WallpaperApplyToMonitor(String, String),
    
    // 缩略图
    ThumbnailLoaded(String, Result<Handle, String>),
    
    // 设置
    SettingsChanged(SettingsChange),
    AddFolder,
    RemoveFolder(PathBuf),
    
    // 后台任务
    LibraryScanComplete(Vec<WallpaperItem>),
    EngineEvent(EngineEvent),
}

#[derive(Debug, Clone)]
enum View {
    Library,
    Settings,
}

enum ThumbnailState {
    Loading,
    Loaded(iced::widget::image::Handle),
    Failed,
}
```

### 4.2 Views 结构 (iced 声明式)
```rust
// views/library.rs
use iced::widget::{column, row, text_input, scrollable, container};
use iced::{Element, Length};

impl WayvidApp {
    pub fn view_library(&self) -> Element<Message> {
        let search_bar = text_input("Search wallpapers...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .width(Length::Fill);
        
        // 壁纸网格 (lazy 加载)
        let grid = self.wallpapers
            .chunks(4)  // 4 列
            .map(|row_items| {
                row(row_items.iter().map(|wp| self.wallpaper_card(wp)))
                    .spacing(16)
                    .into()
            })
            .collect::<Vec<_>>();
        
        let content = scrollable(
            column(grid).spacing(16).padding(20)
        ).height(Length::Fill);
        
        column![search_bar, content]
            .spacing(10)
            .into()
    }
}

// views/settings.rs
impl WayvidApp {
    pub fn view_settings(&self) -> Element<Message> {
        let autostart = checkbox(
            "开机自启动",
            self.settings.autostart,
            |v| Message::SettingsChanged(SettingsChange::Autostart(v))
        );
        
        let minimize = checkbox(
            "关闭时最小化到托盘",
            self.settings.minimize_to_tray,
            |v| Message::SettingsChanged(SettingsChange::MinimizeToTray(v))
        );
        
        let folders_list = column(
            self.settings.wallpaper_folders.iter().map(|folder| {
                row![
                    text(folder.display().to_string()),
                    button("移除").on_press(Message::RemoveFolder(folder.clone()))
                ].spacing(10).into()
            })
        );
        
        let add_folder_btn = button("添加文件夹")
            .on_press(Message::AddFolder);
        
        column![
            text("基本设置").size(24),
            autostart,
            minimize,
            text("壁纸文件夹").size(24),
            folders_list,
            add_folder_btn,
        ]
        .spacing(15)
        .padding(20)
        .into()
    }
}
```

### 4.3 Widgets (iced 组件)
```rust
// widgets/wallpaper_card.rs
use iced::widget::{button, column, container, image, text};
use iced::{Element, Length, Theme};

impl WayvidApp {
    pub fn wallpaper_card(&self, wallpaper: &WallpaperItem) -> Element<Message> {
        let thumbnail: Element<Message> = match self.thumbnails.get(&wallpaper.id) {
            Some(ThumbnailState::Loaded(handle)) => {
                image(handle.clone())
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(112.0))
                    .into()
            }
            Some(ThumbnailState::Loading) => {
                container(text("加载中..."))
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(112.0))
                    .center_x()
                    .center_y()
                    .into()
            }
            _ => {
                container(text("🖼️"))
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(112.0))
                    .center_x()
                    .center_y()
                    .into()
            }
        };
        
        let type_badge = match wallpaper.wallpaper_type {
            WallpaperType::Scene => text("🎬 Scene").size(12),
            WallpaperType::Video => text("🎥 Video").size(12),
            WallpaperType::Gif => text("🌟 GIF").size(12),
            WallpaperType::Image => text("🖼️ Image").size(12),
        };
        
        let card_content = column![
            thumbnail,
            text(&wallpaper.name).size(14),
            type_badge,
        ]
        .spacing(4)
        .width(Length::Fixed(200.0));
        
        button(card_content)
            .on_press(Message::WallpaperSelected(wallpaper.id.clone()))
            .padding(8)
            .style(theme::Button::Secondary)
            .into()
    }
}

// 双击应用和右键菜单通过 iced 的 mouse_area 和 overlay 实现
```

## 5. wayvid-ctl (精简 CLI)

```rust
#[derive(Parser)]
#[command(name = "wayvid-ctl")]
#[command(about = "wayvid 命令行控制工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 应用壁纸
    Apply {
        /// 壁纸路径
        path: String,
        /// 目标显示器 (可选，默认全部)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 暂停壁纸
    Pause,
    /// 恢复壁纸
    Resume,
    /// 显示状态
    Status {
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
}
```

## 6. 文件路径

```
~/.config/wayvid/
├── settings.yaml       # 应用设置 (GUI 自动管理)
└── config.yaml         # 旧格式 (向后兼容，可选)

~/.cache/wayvid/
├── library.db          # SQLite 壁纸索引
└── thumbnails/
    ├── <hash>.webp     # 缩略图
    └── <hash>@2x.webp  # HiDPI 缩略图

~/.local/share/wayvid/
└── logs/               # 日志文件 (可选)
```

## 7. IPC 协议 (简化)

```rust
#[derive(Serialize, Deserialize)]
pub enum IpcCommand {
    Apply { source: String, output: Option<String> },
    Pause { output: Option<String> },
    Resume { output: Option<String> },
    GetStatus,
    Quit,
}

#[derive(Serialize, Deserialize)]
pub enum IpcResponse {
    Ok { data: Option<Value> },
    Error { message: String },
}
```

## 8. 性能目标

| 指标 | 目标 |
|-----|------|
| 启动时间 (GUI) | < 1s |
| 首帧渲染 | < 2s |
| 缩略图加载 (已缓存) | < 50ms |
| 缩略图生成 | < 500ms |
| 内存占用 (空闲) | < 100MB |
| 内存占用 (4K视频) | < 300MB |
| CPU 占用 (播放中) | < 5% |
