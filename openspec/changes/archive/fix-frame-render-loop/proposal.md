# Change: Fix Frame Render Loop Architecture

## Why
当前的渲染循环完全依赖 Wayland frame callback 来驱动，这导致了两个严重问题：

1. **帧回调链断裂**：当 FPS throttle 或其他原因导致跳过渲染时，没有 `commit()` 发生，Wayland frame callback 无法触发，整个渲染循环停止。

2. **mpv 更新回调未使用**：mpv 的 `mpv_render_context_set_update_callback` 会在新帧可用时通知我们，但当前代码只是设置了一个 flag，没有用它来驱动渲染。

参考 mpvpaper 的实现，正确的架构应该是：
- mpv update callback 通知"有新帧可用"
- 主循环轮询检查是否需要渲染
- Wayland frame callback 用于 vsync 同步，但不是唯一的渲染触发器

## What Changes
- **MODIFIED**: 渲染循环从"事件驱动"改为"轮询驱动"
- **MODIFIED**: mpv update callback 用于唤醒主循环
- **MODIFIED**: Wayland frame callback 仅用于 vsync，不再是渲染的唯一驱动
- **REMOVED**: FPS throttle 通过跳过渲染实现（改为通过 mpv 播放速度控制）

## Impact
- Affected specs: `video-playback`, `wayland-backend`
- Affected code: `src/backend/wayland/app.rs`, `src/video/mpv.rs`, `src/video/shared_decode.rs`
