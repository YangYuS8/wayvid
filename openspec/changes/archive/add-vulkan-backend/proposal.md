# Change: Add Vulkan Rendering Backend Support

## Why
目前 wayvid 仅支持 OpenGL (EGL) 作为渲染后端，限制了在以下场景中的使用：

1. **性能优化**：Vulkan 提供更低的 CPU 开销和更好的多线程支持
2. **HDR 渲染**：Vulkan 对 HDR 格式（如 scRGB、HDR10）有更好的原生支持
3. **未来兼容性**：许多新的 Wayland 合成器（如 wlroots 0.18+）推荐使用 Vulkan
4. **GPU 利用率**：Vulkan 允许更精细的资源管理和更好的 GPU 利用

## What Changes

### Phase 1: Vulkan 基础设施 (MVP)
- 添加 Vulkan 实例创建和设备选择
- 实现 Wayland-Vulkan 表面 (VK_KHR_wayland_surface)
- 基础纹理上传和渲染管线
- 与现有配置系统集成（渲染后端选择）

### Phase 2: 功能完善
- 实现与 OpenGL 相同的图层渲染功能
- 支持混合模式（Normal, Additive, Multiply 等）
- 支持变换（缩放、旋转、平移）
- 实现视频帧上传和显示

### Phase 3: 高级功能 (可选/未来)
- HDR 渲染支持（scRGB, HDR10）
- 计算着色器效果
- 异步资源上传
- 多 GPU 支持

## Impact

### 代码变更
- 新增 `src/video/vulkan/` 模块：Vulkan 上下文和渲染
  - `mod.rs` - 模块入口
  - `instance.rs` - Vulkan 实例管理
  - `device.rs` - 设备选择和队列管理
  - `surface.rs` - Wayland 表面集成
  - `pipeline.rs` - 渲染管线
  - `texture.rs` - 纹理管理
- 修改 `src/video/mod.rs`：添加 vulkan 模块
- 修改 `src/backend/wayland/surface.rs`：支持 Vulkan 渲染路径
- 修改 `src/config/types.rs`：添加渲染后端配置选项

### 依赖变更
- 添加 `ash` crate (Vulkan bindings)
- 添加 `ash-window` crate (Wayland surface 集成)
- 可选：`gpu-allocator` (内存分配器)

### 兼容性
- **非破坏性变更**：OpenGL 渲染保持默认，Vulkan 为可选
- 通过 feature flag `backend-vulkan` 控制
- 配置文件支持 `renderer: "vulkan" | "opengl" | "auto"`

### 风险
- Vulkan 驱动兼容性（部分旧 GPU 不支持）
- 需要 Vulkan 1.0+ 支持
- 初期可能存在性能调优需求

## Success Criteria
- 能够使用 Vulkan 渲染视频壁纸和场景壁纸
- 渲染质量与 OpenGL 后端一致
- 性能至少与 OpenGL 持平（预期更优）
- 在 AMD/NVIDIA/Intel GPU 上测试通过
- 不影响现有 OpenGL 渲染功能
