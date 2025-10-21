# wayvid M2 开发进度

**基线**: M1 MVP ✅ (已完成 Wayland 后端基础功能)  
**目标**: OpenGL/EGL 视频渲染  
**开始日期**: 2025年10月21日

---

## 任务列表

### ✅ 已完成

#### M1 基础 (2025-10-20)
- [x] Wayland 连接与 registry 绑定
- [x] Layer Shell surface 创建
- [x] wl_output 管理
- [x] Configure event 处理 (修复循环bug)
- [x] 配置系统 (YAML 解析)
- [x] 布局算法 (calculate_layout)
- [x] 在 Hyprland 0.51.0 上验证
- [x] 创建 M1 测试报告

#### M2 Phase 1 - 框架搭建 (2025-10-21)
- [x] 创建 M2 开发计划 (M2_PLAN.md)
- [x] 创建 EGL 模块 (src/video/egl.rs)
- [x] 添加 wayland-egl 依赖
- [x] 定义 EglContext 和 EglWindow 结构
- [x] 编译验证通过

### 🔄 进行中

#### M2 Phase 1 - EGL 实现 (预计 3-5 天)
- [x] **实现 EglContext::new()** ✅
  - [x] 添加 khronos-egl bindings
  - [x] 实现 eglGetDisplay(wl_display)
  - [x] 实现 eglInitialize
  - [x] 实现 eglChooseConfig (RGBA8888, depth 24, stencil 8)
  - [x] 实现 eglCreateContext (OpenGL 3.0 Core Profile)
  - [x] 添加错误处理
  
- [x] **实现 EglContext::create_window()** ✅
  - [x] wayland-egl WlEglSurface binding
  - [x] eglCreateWindowSurface
  - [x] 验证 surface 创建

- [x] **实现渲染操作** ✅
  - [x] make_current()
  - [x] swap_buffers()
  - [x] resize() for EglWindow
  - [x] 清理逻辑 (Drop)

- [x] **集成到 WaylandSurface** ✅
  - [x] 在 AppState 中添加全局 EglContext
  - [x] 从 Connection 获取 wl_display 指针
  - [x] 在 surface.rs 中添加 EglWindow 字段
  - [x] 在 configure() 中初始化 EGL window
  - [x] 在 render() 中调用 make_current() 和 swap_buffers()
  - [x] 测试 OpenGL 上下文创建

- [x] **验证 EGL 功能** ✅
  - [x] EGL 1.5 初始化成功
  - [x] EGL context 创建成功 (OpenGL 3.0 Core)
  - [x] EGL window 创建成功 (2160x1440)
  - [x] make_current() 和 swap_buffers() 正常工作
  - [x] 在 Hyprland 上稳定运行，无错误

### ⏳ 待办

#### M2 Phase 2 - mpv 集成 (预计 1-2 周)
- [ ] **解决 libmpv 版本问题**
  - [ ] 选项A: 使用 libmpv-sys 直接 FFI
  - [ ] 选项B: 切换到 GStreamer
  - [ ] 重新启用 video 初始化代码
  
- [ ] **mpv_render_context 创建**
  - [ ] 初始化 OpenGL render context
  - [ ] 实现 get_proc_address 回调
  - [ ] 绑定到 EGL context

- [ ] **实现帧渲染**
  - [ ] mpv_render_context_render()
  - [ ] FBO 绑定
  - [ ] 纹理上传

#### M2 Phase 3 - 帧同步 (预计 1 周)
- [ ] **wl_callback 集成**
  - [ ] 实现 Dispatch<wl_callback::WlCallback>
  - [ ] wl_surface::frame() 请求
  - [ ] 渲染循环同步

- [ ] **FPS 限制**
  - [ ] 读取 PowerConfig.max_fps
  - [ ] 实现帧率限制器
  - [ ] 测试不同 FPS 设置

#### M2 Phase 4 - 布局应用 (预计几天)
- [ ] **OpenGL viewport 设置**
  - [ ] 使用 calculate_layout() 结果
  - [ ] 设置 glViewport()
  - [ ] 纹理坐标变换

- [ ] **测试所有布局模式**
  - [ ] Fill (裁剪)
  - [ ] Contain (留黑边)
  - [ ] Stretch (拉伸)
  - [ ] Cover (覆盖)
  - [ ] Centre (居中)

#### M2 Phase 5 - 多输出支持 (预计 1 周)
- [ ] **xdg-output 协议**
  - [ ] 添加 xdg_output_manager 绑定
  - [ ] 获取输出名称和描述
  
- [ ] **热插拔处理**
  - [ ] 监听 global_remove 事件
  - [ ] 动态创建/销毁 surface
  - [ ] 清理 EGL 资源
  - [ ] 测试插拔场景

#### M2 Phase 6 - 电源管理 (预计几天)
- [ ] **DPMS 检测**
  - [ ] 跟踪输出电源状态
  - [ ] pause_when_hidden 实现
  
- [ ] **电池检测**
  - [ ] 读取 /sys/class/power_supply
  - [ ] pause_on_battery 实现
  
- [ ] **性能优化**
  - [ ] 应用 max_fps
  - [ ] CPU/GPU 优化

#### M2 Phase 7 - 测试与文档 (预计 1 周)
- [ ] **功能测试**
  - [ ] 单显示器场景
  - [ ] 多显示器场景
  - [ ] 热插拔稳定性
  - [ ] 长时间运行 (24h+)
  
- [ ] **性能测试**
  - [ ] CPU 占用率
  - [ ] 内存使用
  - [ ] GPU 使用率
  - [ ] 帧率稳定性
  
- [ ] **兼容性测试**
  - [ ] Hyprland
  - [ ] Sway
  - [ ] niri
  - [ ] 其他 wlroots compositors
  
- [ ] **文档完善**
  - [ ] M2_DELIVERY_REPORT.md
  - [ ] README 更新
  - [ ] 配置示例
  - [ ] 故障排除指南

---

## 里程碑

| 阶段 | 目标 | 预计完成 | 状态 |
|------|------|----------|------|
| **M2.1** | EGL 上下文初始化 | Week 1 | 🔄 进行中 |
| **M2.2** | mpv 渲染集成 | Week 3 | ⏳ 待办 |
| **M2.3** | 帧同步 | Week 4 | ⏳ 待办 |
| **M2.4** | 布局应用 | Week 4 | ⏳ 待办 |
| **M2.5** | 多输出支持 | Week 5 | ⏳ 待办 |
| **M2.6** | 电源管理 | Week 6 | ⏳ 待办 |
| **M2.7** | 测试与交付 | Week 7 | ⏳ 待办 |

---

## 当前焦点

**刚完成**: EGL 集成到 Wayland Surface ✅  
**下一步**: OpenGL 渲染测试 或 mpv_render_context 集成  
**阻塞项**: libmpv 版本冲突 (VersionMismatch)

---

## 技术笔记

### EGL 初始化参考代码
```rust
// 1. Get EGL display from Wayland
let egl_display = unsafe {
    egl::get_display(wl_display as *mut _)
        .ok_or_else(|| anyhow!("Failed to get EGL display"))?
};

// 2. Initialize EGL
let (major, minor) = unsafe {
    let mut major = 0;
    let mut minor = 0;
    egl::initialize(egl_display, &mut major, &mut minor)
        .map_err(|_| anyhow!("Failed to initialize EGL"))?;
    (major, minor)
};

// 3. Choose config
let config_attribs = [
    egl::SURFACE_TYPE, egl::WINDOW_BIT,
    egl::RENDERABLE_TYPE, egl::OPENGL_BIT,
    egl::RED_SIZE, 8,
    egl::GREEN_SIZE, 8,
    egl::BLUE_SIZE, 8,
    egl::ALPHA_SIZE, 8,
    egl::NONE,
];

let mut configs = vec![std::ptr::null(); 1];
let mut num_configs = 0;
unsafe {
    egl::choose_config(
        egl_display,
        config_attribs.as_ptr(),
        configs.as_mut_ptr(),
        1,
        &mut num_configs,
    ).map_err(|_| anyhow!("Failed to choose EGL config"))?;
}

// 4. Bind OpenGL API
unsafe {
    egl::bind_api(egl::OPENGL_API)
        .map_err(|_| anyhow!("Failed to bind OpenGL API"))?;
}

// 5. Create context
let context_attribs = [
    egl::CONTEXT_MAJOR_VERSION, 3,
    egl::CONTEXT_MINOR_VERSION, 0,
    egl::NONE,
];

let egl_context = unsafe {
    egl::create_context(
        egl_display,
        configs[0],
        egl::NO_CONTEXT,
        context_attribs.as_ptr(),
    ).map_err(|_| anyhow!("Failed to create EGL context"))?
};
```

### 已知问题

1. **libmpv 版本冲突** (M1遗留)
   - 错误: `VersionMismatch { linked: 65644, loaded: 131077 }`
   - 计划: M2.2 使用 libmpv-sys 或 GStreamer
   - 影响: 暂时无法测试视频播放

2. **wayland-egl API**
   - 需要 FFI bindings: `wl_egl_window_create`, `wl_egl_window_resize`, `wl_egl_window_destroy`
   - wayland-egl crate 提供这些绑定
   - 文档: https://docs.rs/wayland-egl

---

---

## 🎉 M2 Phase 1 完成！

**完成日期**: 2025-10-21

### 测试结果

```
✅ EGL 初始化: EGL 1.5
✅ EGL context: OpenGL 3.0 Core Profile
✅ EGL window: 2160x1440 (output-61)
✅ make_current(): 正常工作
✅ swap_buffers(): 正常工作
✅ Hyprland 集成: Layer surface 可见
✅ 稳定性: 无崩溃，无错误
```

### 测试日志摘录

```
INFO wayvid::video::egl: EGL initialized: 1.5
INFO wayvid::video::egl: EGL context created successfully
INFO wayvid::backend::wayland::app:   ✓ EGL context initialized
INFO wayvid::backend::wayland::surface:   ✓ EGL window created for output output-61
```

### Hyprland 验证

```bash
$ hyprctl layers | grep wayvid
Layer 556b3b55da10: xywh: 1639 1437 2160 1440, namespace: wayvid, pid: 394855
```

---

**最后更新**: 2025-10-21  
**当前进度**: M2 Phase 1 完成 ✅ - 准备进入 Phase 2 (mpv 集成)
