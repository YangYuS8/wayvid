# Change: Add Scene Wallpaper Support for Wallpaper Engine

## Why
Wallpaper Engine 的场景类壁纸 (Scene type) 是最受欢迎的壁纸类型之一，占 Workshop 内容的很大比例。目前 wayvid 只支持视频类壁纸，无法使用场景类壁纸，限制了用户的选择范围。

场景类壁纸使用 JSON 配置描述场景元素（图层、粒子、动画等），需要运行时渲染而非简单的视频播放。

## What Changes

### Phase 1: 基础场景渲染
- 解析场景 `project.json` 中的场景配置
- 支持基础图层类型：图片图层 (image layer)
- 支持图层属性：位置、大小、旋转、透明度
- 使用 OpenGL 渲染场景到 Wayland surface

### Phase 2: 动画系统
- 支持基础动画：滚动 (scroll)、缩放 (scale)、旋转 (spin)
- 支持动画曲线：线性、缓入缓出
- 支持图层混合模式：正常、叠加、相乘

### Phase 3: 高级效果 (可选/未来)
- 粒子系统
- 着色器效果
- 音频响应
- 用户属性 (user properties)

## Impact

### 代码变更
- 新增 `src/we/scene/` 模块：场景解析和渲染
- 修改 `src/we/parser.rs`：扩展支持场景类型
- 修改 `src/video/egl.rs`：添加场景渲染管线
- 新增着色器代码：`src/video/shaders/scene_*.glsl`

### 依赖变更
- 可能需要添加 `image` crate 用于加载场景图片资源

### 兼容性
- **非破坏性变更**：现有视频壁纸功能不受影响
- 场景渲染是可选功能，默认启用

### 风险
- 场景格式复杂，完整兼容需要大量工作
- Phase 1 仅支持基础功能，部分场景可能显示不完整
- 性能需要优化，场景渲染比视频播放更消耗 GPU

## Success Criteria
- 能够加载并显示基础场景壁纸（静态图层 + 简单动画）
- 场景渲染帧率稳定在 60 FPS
- 不影响现有视频壁纸功能
