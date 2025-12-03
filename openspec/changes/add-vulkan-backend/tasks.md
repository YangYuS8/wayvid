## 1. 研究与设计
- [x] 1.1 研究 Vulkan + Wayland 集成最佳实践
- [x] 1.2 设计 Vulkan 渲染架构（与现有 EGL 架构对照）
- [x] 1.3 确定 Phase 1 支持的功能范围
- [x] 1.4 调研 ash crate 和相关依赖

## 2. Vulkan 基础设施 (src/video/vulkan/)
- [x] 2.1 添加 Cargo 依赖 (ash, ash-window)
- [x] 2.2 创建 Vulkan 实例管理 (`instance.rs`)
  - 创建 VkInstance 和调试回调
  - 查询所需扩展 (VK_KHR_wayland_surface)
- [x] 2.3 实现物理设备选择 (`device.rs`)
  - 枚举 GPU 并选择最佳设备
  - 创建逻辑设备和队列
- [x] 2.4 实现 Wayland 表面集成 (`surface.rs`)
  - 创建 VkSurfaceKHR from Wayland surface
  - 实现交换链管理
- [x] 2.5 创建渲染管线 (`pipeline.rs`)
  - 编写顶点/片段着色器 (SPIR-V) - 占位符
  - 创建图形管线
  - 实现描述符集布局

## 3. 纹理和渲染
- [x] 3.1 实现纹理管理 (`texture.rs`)
  - RGBA 纹理上传
  - 图像视图和采样器
- [x] 3.2 实现命令缓冲区录制 (`command.rs`)
  - 渲染通道
  - 绘制命令
- [x] 3.3 实现帧同步 (`sync.rs`)
  - 信号量和栅栏
  - 双缓冲/三缓冲

## 4. 集成
- [x] 4.1 创建 `VulkanContext` 结构（与 `EglContext` 接口对应）
- [x] 4.2 创建 `VulkanWindow` 结构（与 `EglWindow` 接口对应）
- [x] 4.3 修改 `WaylandSurface` 支持 Vulkan 渲染路径（框架已完成，渲染集成待定）
- [x] 4.4 添加配置选项选择渲染后端
- [x] 4.5 实现自动检测（auto 模式：框架已完成，Vulkan 优先逻辑待完善）

## 5. Scene 渲染支持
- [x] 5.1 为 SceneRenderer 添加 Vulkan 支持
- [x] 5.2 实现 Vulkan 着色器（移植现有 GLSL）
- [x] 5.3 实现混合模式（Vulkan blend states）

## 6. 测试与文档
- [ ] 6.1 单元测试：Vulkan 上下文创建
- [ ] 6.2 集成测试：视频渲染
- [ ] 6.3 集成测试：场景渲染
- [ ] 6.4 性能对比测试（Vulkan vs OpenGL）
- [x] 6.5 更新用户文档（配置说明）
- [x] 6.6 更新 CHANGELOG
