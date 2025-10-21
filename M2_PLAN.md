# wayvid M2 开发计划

**基础**: M1 MVP ✅ 完成  
**目标**: 完整 OpenGL/EGL 视频渲染  
**预计时间**: 3-5 周

---

## M2 核心任务

### Phase 1: EGL 上下文 (1周)
- [ ] 创建 `src/video/egl.rs` - EGL 上下文管理
- [ ] 实现 `EglContext::new()` - 创建 EGLDisplay, EGLConfig, EGLContext
- [ ] 实现 `create_window_surface()` - 绑定 wl_egl_window
- [ ] 测试 OpenGL 上下文初始化

### Phase 2: mpv 渲染集成 (1-2周)
- [ ] 修复 libmpv 版本问题 (重新编译或使用 sys binding)
- [ ] 初始化 `mpv_render_context` with OpenGL params
- [ ] 实现 `get_proc_address` 回调
- [ ] 实现 FBO 渲染到 wl_egl_window
- [ ] 测试单帧渲染

### Phase 3: 帧同步 (1周)
- [ ] 实现 `wl_surface::frame()` 回调
- [ ] 添加 `wl_callback` Dispatch
- [ ] 集成 mpv render + swap buffers
- [ ] 实现 FPS 限制器
- [ ] 测试 vsync 和流畅播放

### Phase 4: 布局应用 (几天)
- [ ] 使用 `calculate_layout()` 结果
- [ ] 设置 OpenGL viewport
- [ ] 实现纹理坐标变换
- [ ] 测试所有 5 种布局模式

### Phase 5: 多输出与热插拔 (1周)
- [ ] 添加 xdg-output 协议
- [ ] 实现 `global_remove` 处理
- [ ] 动态创建/销毁 surface
- [ ] 测试热插拔场景

### Phase 6: 电源管理 (几天)
- [ ] DPMS 状态检测
- [ ] 实现 pause/resume 逻辑
- [ ] 电池状态检测 (/sys/class/power_supply)
- [ ] 应用 max_fps 限制

---

## 当前状态

### ✅ 已完成 (M1)
- Wayland 连接与 registry
- Layer Shell 集成
- wl_output 管理
- Surface 创建与配置
- 类型系统 (VideoSource, LayoutMode, OutputInfo)
- 配置系统 (YAML, per-output overrides)
- 布局算法 (calculate_layout + tests)
- CLI 工具 (run, check)
- 文档 (9 个 markdown)

### 🟡 部分完成 (需 M2 完善)
- MPV 播放器 (结构存在，版本冲突未解决)
- EGL 类型 (定义存在，未实现)
- Render 函数 (占位符)

### ❌ 待实现 (M2)
- EGL 上下文管理
- mpv_render_context 集成
- OpenGL 渲染循环
- 帧同步 (wl_callback)
- 热插拔支持
- 电源管理

---

## 技术要点

### EGL 初始化流程
```rust
// 1. 获取 EGL display
let egl_display = eglGetDisplay(wl_display);
eglInitialize(egl_display, ...);

// 2. 选择 config
let config_attribs = [
    EGL_SURFACE_TYPE, EGL_WINDOW_BIT,
    EGL_RENDERABLE_TYPE, EGL_OPENGL_BIT,
    EGL_RED_SIZE, 8,
    EGL_GREEN_SIZE, 8,
    EGL_BLUE_SIZE, 8,
    EGL_ALPHA_SIZE, 8,
    EGL_NONE,
];
let mut config = null();
eglChooseConfig(egl_display, config_attribs, &mut config, 1, ...);

// 3. 创建 context
eglBindAPI(EGL_OPENGL_API);
let context_attribs = [EGL_CONTEXT_MAJOR_VERSION, 3, EGL_CONTEXT_MINOR_VERSION, 0, EGL_NONE];
let egl_context = eglCreateContext(egl_display, config, EGL_NO_CONTEXT, context_attribs);

// 4. 创建 window surface
let egl_window = wl_egl_window_create(wl_surface, width, height);
let egl_surface = eglCreateWindowSurface(egl_display, config, egl_window, null());

// 5. Make current
eglMakeCurrent(egl_display, egl_surface, egl_surface, egl_context);
```

### mpv 渲染流程
```rust
// 1. 初始化 render context
let render_params = [
    MPV_RENDER_PARAM_API_TYPE, "opengl",
    MPV_RENDER_PARAM_OPENGL_INIT_PARAMS, &opengl_init_params,
];
mpv_render_context_create(&mut ctx, mpv, render_params);

// 2. 每帧渲染
let fbo_params = [
    MPV_RENDER_PARAM_OPENGL_FBO, &fbo_data,
    MPV_RENDER_PARAM_FLIP_Y, 1,
];
mpv_render_context_render(ctx, fbo_params);

// 3. Swap buffers
eglSwapBuffers(egl_display, egl_surface);

// 4. 请求下一帧
wl_surface::frame(qh, ...);
```

---

## 依赖更新

需要添加:
```toml
[dependencies]
wayland-egl = "0.32"  # wl_egl_window 绑定
libmpv-sys = "4.0"  # 直接 FFI，跳过版本检查
```

---

## 测试策略

### 单元测试
- EGL 初始化 (需 mock)
- 布局计算 (已有)
- 配置解析 (已有)

### 集成测试
- Surface 创建
- 视频加载
- 多输出管理

### 手动测试
- Hyprland + niri 兼容性
- 热插拔稳定性
- 长时间运行
- 内存泄漏检查

---

## 风险与缓解

| 风险 | 影响 | 缓解方案 |
|------|------|----------|
| libmpv 版本冲突 | 高 | 使用 libmpv-sys 直接 FFI |
| EGL 错误难调试 | 中 | 详细错误日志 + 文档 |
| 性能不足 | 中 | 硬件解码 + FPS 限制 |
| 热插拔竞态 | 低 | 互斥锁 + 测试 |

---

## 下一步

**立即开始**: Phase 1 - EGL 上下文实现  
**文件**: `src/video/egl.rs`  
**目标**: 创建 EGL display, context, surface 并测试 OpenGL 初始化

**预计完成**: 3-5 天

---

**创建日期**: 2025年10月21日  
**状态**: M1 ✅ → M2 开始
