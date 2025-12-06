# Tasks: Fix GUI Desktop Integration

## Phase 1: Theme Persistence (P0)

- [x] **1.1** 修改 `Message::ToggleTheme` 处理逻辑，添加设置保存
  - 文件: `crates/wayvid-gui/src/app.rs`
  - 在主题切换时更新 `state.app_settings.gui.theme`
  - 调用 `trigger_settings_save()`

- [x] **1.2** 验证主题设置持久化
  - 启动应用，切换主题
  - 关闭并重新启动
  - 确认主题保持不变

## Phase 2: Minimize to Tray Implementation (P2)

- [x] **2.1** 调研 iced 0.13 Wayland 窗口关闭事件
  - 检查 `iced::event::Event::Window(CloseRequested)` 支持
  - 使用 `event::listen()` 订阅窗口事件

- [x] **2.2** 实现窗口关闭拦截
  - 添加 `WindowCloseRequested` 消息类型
  - 订阅 window events
  - 在 `minimize_to_tray` 启用时使用 `window::minimize`
  - 否则正常关闭窗口

## Phase 3: Autostart Optimization (P1)

- [x] **3.1** 验证当前 XDG autostart 功能
  - 功能已存在于 GUI 设置中
  - 写入 `~/.config/autostart/wayvid.desktop`

- [x] **3.2** 更新自启动文档
  - 添加 niri `spawn-at-startup` 示例
  - 添加 Hyprland `exec-once` 示例
  - 添加 Sway `exec` 示例
  - 更新 mdbook 用户指南 (`docs/src/user-guide/installation.md`)

## Phase 4: Documentation & Testing

- [x] **4.1** 更新 CHANGELOG.md

- [ ] **4.2** 测试完整工作流
  - niri + noctalia-shell 环境
  - 主题持久化
  - 最小化功能 (需要用户手动测试)

## Completion Criteria

1. ✅ 主题设置在应用重启后保持
2. ✅ 最小化到托盘功能实现（使用 window::minimize）
3. ✅ 自启动功能有完善文档
4. ✅ 所有自动化测试通过
