## Context

Wallpaper Engine 场景类壁纸使用 JSON 配置文件描述场景结构，包含多个图层和动画效果。与视频壁纸不同，场景需要实时渲染而非解码播放。

### 场景格式概述

场景 `project.json` 示例结构：
```json
{
  "type": "scene",
  "general": {
    "properties": { ... }
  },
  "objects": [
    {
      "id": 1,
      "image": "materials/layer1.png",
      "origin": "0.5 0.5",
      "scale": "1 1",
      "angles": "0 0 0",
      "visible": true,
      "effects": [ ... ]
    }
  ]
}
```

### 约束
- Wallpaper Engine 场景格式未公开文档，需要逆向工程
- 完整兼容需要支持大量效果类型，Phase 1 仅支持子集
- 性能敏感：场景渲染必须保持 60 FPS

## Goals / Non-Goals

### Goals
- 支持加载和显示基础场景壁纸
- 支持图片图层和基础变换
- 支持简单动画效果
- 与现有架构无缝集成

### Non-Goals (Phase 1)
- 粒子系统
- 自定义着色器效果
- 音频响应
- 3D 效果
- 用户属性编辑器

## Decisions

### 1. 渲染方式
**决定**: 使用 OpenGL 2D 渲染，复用现有 EGL 上下文

**理由**: 
- 场景本质是 2D 图层合成
- 复用现有渲染基础设施
- 简化实现复杂度

**替代方案**: 
- wgpu：更现代但增加依赖
- Skia：功能强大但体积大

### 2. 资源加载
**决定**: 使用 `image` crate 加载图片，转为 OpenGL 纹理

**理由**:
- `image` 已在项目依赖中
- 支持常见格式 (PNG, JPEG)
- 性能足够

### 3. 动画系统
**决定**: 基于时间的动画更新，每帧计算属性

**理由**:
- 简单直接
- 易于实现插值
- 与 Wayland frame callback 配合良好

## Module Structure

```
src/we/scene/
├── mod.rs           # 模块入口
├── types.rs         # 场景数据类型定义
├── parser.rs        # JSON 解析器
├── layer.rs         # 图层处理
├── animation.rs     # 动画系统
└── renderer.rs      # OpenGL 渲染器

src/video/shaders/
├── scene_layer.vert # 图层顶点着色器
└── scene_layer.frag # 图层片段着色器
```

## Risks / Trade-offs

### 风险 1: 场景格式不兼容
**问题**: 部分场景使用未支持的特性
**缓解**: 优雅降级，显示警告但继续渲染可支持的部分

### 风险 2: 性能问题
**问题**: 复杂场景可能导致帧率下降
**缓解**: 
- 图层缓存
- 脏区域更新
- 提供性能设置选项

### Trade-off: 功能完整性 vs 实现复杂度
选择先实现核心功能（图层 + 基础动画），后续迭代添加高级效果。

## Open Questions

1. 是否需要支持场景中的嵌入视频？
2. 如何处理场景资源的内存管理（大型场景可能有很多图片）？
3. 是否需要场景预览生成功能？
