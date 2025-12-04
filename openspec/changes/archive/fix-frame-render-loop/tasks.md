## 1. 重构渲染循环架构

- [ ] 1.1 移除对 Wayland frame callback 作为唯一渲染驱动的依赖
- [ ] 1.2 实现基于轮询的渲染检查（使用 mpv_render_context_update）
- [ ] 1.3 保留 frame callback 仅用于 vsync 同步

## 2. 改进 mpv 集成

- [ ] 2.1 使用 mpv update callback 唤醒主循环（通过 eventfd 或 pipe）
- [ ] 2.2 在主循环中正确处理 mpv 事件队列
- [ ] 2.3 确保 render context 正确报告 swap

## 3. 简化 FPS 控制

- [ ] 3.1 移除基于跳过渲染的 FPS throttle
- [ ] 3.2 考虑通过 mpv 播放速度控制来实现节能

## 4. 清理调试代码

- [ ] 4.1 移除临时添加的日志
- [ ] 4.2 恢复正常日志级别
