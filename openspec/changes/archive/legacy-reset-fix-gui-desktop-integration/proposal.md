# Fix GUI Desktop Integration Issues

## Summary

修复 wayvid-gui 在 niri + noctalia-shell 等现代 Wayland 环境下的三个桌面集成问题：

1. **最小化到托盘功能失效** - 关闭窗口时无法后台保持运行
2. **主题设置不持久化** - 每次启动都重置为深色模式
3. **开机自启动功能需优化** - 需要支持 niri 的 `spawn-at-startup` 配置方式

## Problem Analysis

### 问题 1: 最小化到托盘失效

**现状分析**:
- `GuiSettings` 中有 `minimize_to_tray` 和 `start_minimized` 字段
- 但 iced 的 Wayland 后端可能没有正确处理窗口关闭请求
- niri + noctalia-shell 环境下，关闭窗口会直接终止进程

**根本原因**:
- iced 在 Wayland 环境下对窗口关闭事件的处理不同于 X11
- 需要订阅 `window::Event::CloseRequested` 并选择性忽略来实现"隐藏到托盘"
- 可能需要使用 `window::minimize()` 或其他方式来隐藏窗口

### 问题 2: 主题设置不持久化

**现状分析**:
- `GuiSettings.theme` 字段存储为 String ("dark"/"light")
- `App::new()` 在启动时正确读取设置并初始化 `theme`
- `Message::ToggleTheme` 切换主题时**没有更新设置并保存**

**修复方案**:
在 `Message::ToggleTheme` 处理中添加设置保存:
```rust
Message::ToggleTheme => {
    self.theme = match self.theme {
        WayvidTheme::Dark => WayvidTheme::Light,
        WayvidTheme::Light => WayvidTheme::Dark,
    };
    // 需要添加：保存主题设置
    self.state.app_settings.gui.theme = match self.theme {
        WayvidTheme::Dark => "dark".to_string(),
        WayvidTheme::Light => "light".to_string(),
    };
    self.trigger_settings_save();
    Task::none()
}
```

### 问题 3: 开机自启动

**现状分析**:
- 当前使用 XDG autostart (.desktop 文件) 方式
- 写入 `~/.config/autostart/wayvid.desktop`
- 使用 `Exec=wayvid-gui --minimized`

**niri 推荐方式**:
根据 niri 官方文档，有两种自启动方式：
1. **XDG autostart** - niri 的 systemd session 支持 `xdg-desktop-autostart`
2. **spawn-at-startup** - 在 `~/.config/niri/config.kdl` 中配置

**优化方案**:
- 保留 XDG autostart 作为通用方式（适用于所有支持 XDG 规范的环境）
- 在文档中说明 niri 用户可以使用 `spawn-at-startup "wayvid-gui" "--minimized"` 方式
- 确保 `--minimized` 参数正确工作

## Proposed Solution

### 修复优先级

1. **P0 - 主题持久化** (简单修复，立即可做)
2. **P1 - 开机自启动文档** (文档更新，确保功能正常)
3. **P2 - 最小化到托盘** (需要深入调研 iced Wayland 行为)

### 详细方案

#### 方案 A: 主题持久化 (必做)

修改 `app.rs` 中 `Message::ToggleTheme` 的处理逻辑，添加设置保存。

#### 方案 B: 最小化到托盘 (需调研)

**选项 B1: 使用 iced window events**
- 订阅 `window::Event::CloseRequested`
- 当 `minimize_to_tray` 启用时，取消关闭操作并最小化窗口
- 需要验证 iced 0.13 在 Wayland 下的支持情况

**选项 B2: 使用系统托盘图标 (long-term)**
- 集成 `ksni` 或 `libappindicator` 实现真正的系统托盘
- 在 noctalia-shell 等面板上显示托盘图标
- 这需要更多工作，可作为后续增强

**选项 B3: 后台服务模式**
- 窗口关闭时不退出，转为纯后台播放
- 通过 IPC 或 DBus 唤醒 GUI
- 这与当前 GUI-first 架构有冲突

**推荐**: 先实现 B1，如果 iced 支持不佳则考虑 B3

#### 方案 C: 自启动优化

- 保留当前 XDG autostart 实现
- 更新文档说明 niri 用户的配置方法
- 确保 `--minimized` 参数正确触发最小化启动

## Impact

- **用户体验**: 显著改善，特别是对 niri 用户
- **代码复杂度**: 低 (主题修复) 到中 (托盘功能)
- **向后兼容**: 完全兼容，仅增强现有功能

## Alternatives Considered

1. **完全移除托盘功能** - 不推荐，用户期望此功能存在
2. **仅支持 niri spawn-at-startup** - 不推荐，会丢失通用性
3. **使用 D-Bus 后台服务** - 过于复杂，留待未来版本

## References

- [niri Configuration: Miscellaneous - spawn-at-startup](https://github.com/YaLTeR/niri/wiki/Configuration:-Miscellaneous#spawn-at-startup)
- [iced Window Management](https://docs.rs/iced/latest/iced/window/index.html)
- [XDG Autostart Specification](https://specifications.freedesktop.org/autostart-spec/latest/)
