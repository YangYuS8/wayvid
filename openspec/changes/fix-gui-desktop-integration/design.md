# Design: GUI Desktop Integration

## Theme Persistence

### Current Implementation

```rust
// app.rs - App::new()
let theme = if state.app_settings.gui.theme == "light" {
    WayvidTheme::Light
} else {
    WayvidTheme::Dark
};

// app.rs - Message::ToggleTheme handler
Message::ToggleTheme => {
    self.theme = match self.theme {
        WayvidTheme::Dark => WayvidTheme::Light,
        WayvidTheme::Light => WayvidTheme::Dark,
    };
    Task::none()  // BUG: 没有保存设置！
}
```

### Fixed Implementation

```rust
Message::ToggleTheme => {
    self.theme = match self.theme {
        WayvidTheme::Dark => WayvidTheme::Light,
        WayvidTheme::Light => WayvidTheme::Dark,
    };
    // 保存主题设置
    self.state.app_settings.gui.theme = match self.theme {
        WayvidTheme::Dark => "dark".to_string(),
        WayvidTheme::Light => "light".to_string(),
    };
    self.trigger_settings_save();
    Task::none()
}
```

## Minimize to Tray

### iced Window Events (调研)

iced 0.13 提供 window event subscription:

```rust
use iced::window;

fn subscription(&self) -> Subscription<Message> {
    window::events().map(|(id, event)| Message::WindowEvent(id, event))
}
```

可能的事件类型:
- `window::Event::CloseRequested` - 窗口关闭请求
- `window::Event::Moved` - 窗口移动
- `window::Event::Resized` - 窗口大小改变

### Wayland 限制

在 Wayland 上，应用程序无法:
1. 隐藏窗口到系统托盘（除非有托盘协议支持）
2. 最小化窗口到后台（compositor 可能不支持 minimize 概念）
3. 阻止窗口关闭（compositor 有最终决定权）

### 可行方案

#### 方案 1: 使用 iced window 控制

```rust
// 订阅窗口事件
fn subscription(&self) -> Subscription<Message> {
    let window_events = window::events().map(Message::WindowEvent);
    Subscription::batch([other_subs, window_events])
}

// 处理关闭请求
Message::WindowEvent(id, window::Event::CloseRequested) => {
    if self.state.app_settings.gui.minimize_to_tray {
        // 尝试最小化而非关闭
        window::minimize(id)
    } else {
        window::close(id)
    }
}
```

#### 方案 2: 后台服务模式

如果窗口控制不可行，可以:
1. 窗口关闭时，保持 engine 在后台运行
2. 通过 `wayvid-ctl` 或 IPC 重新打开 GUI
3. 这需要重构为 GUI 可重入模式

### niri 环境特殊考虑

niri 是 scrolling window manager，不支持传统的"最小化"概念。在 niri 上:
- 关闭窗口 = 终止应用
- 没有系统托盘（依赖 panel 如 noctalia-shell）
- 推荐使用 `spawn-at-startup` 自动启动

## Autostart

### 当前实现 (XDG Autostart)

```rust
// settings.rs - AutostartManager
const DESKTOP_ENTRY: &'static str = r#"[Desktop Entry]
Type=Application
Name=Wayvid
Comment=Animated wallpaper manager for Wayland
Exec=wayvid-gui --minimized
Icon=wayvid
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
"#;

pub fn autostart_file() -> PathBuf {
    dirs::config_dir()
        .join("autostart")
        .join("wayvid.desktop")
}
```

### niri 配置方式

niri 用户可以在 `~/.config/niri/config.kdl` 中添加:

```kdl
spawn-at-startup "wayvid-gui" "--minimized"
```

或使用 shell 命令形式:

```kdl
spawn-sh-at-startup "wayvid-gui --minimized"
```

### 文档建议

在用户指南中添加:

```markdown
## Autostart Configuration

### Method 1: GUI Settings (Recommended)
Enable "Autostart" in Settings > General. This creates an XDG autostart entry.

### Method 2: niri spawn-at-startup
For niri users, add to `~/.config/niri/config.kdl`:
```kdl
spawn-at-startup "wayvid-gui" "--minimized"
```

### Method 3: Hyprland exec-once
For Hyprland users, add to `~/.config/hypr/hyprland.conf`:
```conf
exec-once = wayvid-gui --minimized
```
```

## Implementation Order

1. **立即修复**: Theme persistence (简单的代码改动)
2. **调研后决定**: Minimize to tray (取决于 iced Wayland 支持)
3. **文档优先**: Autostart 功能验证和文档更新
