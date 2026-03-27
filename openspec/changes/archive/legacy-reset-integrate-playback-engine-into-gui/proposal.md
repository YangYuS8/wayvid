# Change: Integrate Playback Engine into GUI (v0.5 Architecture)

## Why

v0.5 wayvid 采用 "GUI-first" 设计理念，但当前实现存在以下问题：

1. **守护进程架构已废弃**：GUI 仍然尝试通过 IPC 与外部守护进程通信，但 v0.5 不再使用独立守护进程
2. **引擎启动是空操作**：`start_playback_engine()` 只是占位符，不会启动任何实际播放
3. **壁纸无法显示**：用户应用壁纸时，实际上什么都没发生
4. **状态不一致**：IPC 轮询会覆盖 GUI 手动设置的引擎状态

新方案将播放引擎直接集成到 GUI 进程中，消除外部守护进程依赖，实现真正的单进程架构。

## What Changes

### Architecture Changes (**BREAKING**)

- 移除对外部 `wayvid` 守护进程的依赖
- 将 `wayvid-engine` 集成到 `wayvid-gui` 进程中
- GUI 直接管理 Wayland layer-shell surfaces 和 MPV 播放
- IPC 仅用于 `wayvid-ctl` 外部控制（可选）

### New Components

- `PlaybackEngine`: 高层引擎 API，管理所有输出的壁纸播放
- `WallpaperSession`: 单个输出的播放会话（LayerSurface + MpvPlayer）
- 内置 IPC 服务器：允许 `wayvid-ctl` 远程控制 GUI

### Modified Behavior

- 启动引擎：GUI 内部初始化引擎线程
- 应用壁纸：直接调用引擎 API，立即显示
- 停止引擎：清理所有 layer surfaces 和 MPV 实例
- 显示器检测：引擎直接监听 Wayland output 事件

## Impact

- **Affected specs**: 
  - `gui-integration` - 需要大幅修改
  - `wayland-backend` - 部分修改（集成方式变化）
  - `video-playback` - 无变化（逻辑层不变）
  
- **Affected code**:
  - `crates/wayvid-gui/src/app.rs` - 添加引擎字段和管理
  - `crates/wayvid-gui/src/ipc.rs` - 重构为内置服务器模式
  - `crates/wayvid-engine/src/lib.rs` - 添加 PlaybackEngine API
  - `crates/wayvid-engine/src/wayland/` - 完善 layer-shell 实现

## Benefits

| Before (Daemon Model) | After (Integrated Model) |
|----------------------|--------------------------|
| 两个进程管理复杂 | 单进程，简单直观 |
| IPC 通信开销 | 内部函数调用 |
| 进程同步问题 | 无跨进程同步 |
| systemd 服务依赖 | XDG autostart 即可 |
| 启动/停止需要协调 | 一键启动/停止 |

## Non-Goals

- 本提案不涉及系统托盘功能（将在单独提案中处理）
- 不改变 `wayvid-ctl` 的命令行接口
- 不改变配置文件格式
