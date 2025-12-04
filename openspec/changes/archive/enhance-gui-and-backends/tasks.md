## 1. Vulkan 后端完善性检查
- [x] 1.1 检查 VulkanContext 初始化流程 ✓ 已完成
  - VulkanContext: 完整实现，支持 Wayland surface 创建
- [x] 1.2 检查 VulkanSurface 与 Wayland 集成 ✓ 已完成
  - 支持 VK_KHR_wayland_surface 扩展
- [x] 1.3 检查场景渲染器 VulkanSceneRenderer 实现 ✓ 已完成
  - 已实现基础框架，有变换矩阵和混合模式支持
- [x] 1.4 验证 Vulkan 后端功能完整性 ✓ 已完成
  - check_vulkan_availability() 函数已添加，返回 VulkanInfo

## 2. 渲染后端配置支持
- [x] 2.1 在 config/types.rs 添加 RenderBackend 枚举 ✓ 已有
  - RenderBackend 枚举已存在于 src/core/types.rs
- [x] 2.2 更新配置解析支持 renderer 字段 ✓ 已有
  - config/types.rs 已支持 render_backend 字段
- [x] 2.3 添加后端可用性检测函数 ✓ 已完成
  - check_vulkan_availability() 函数已添加到 video/vulkan/mod.rs

## 3. GUI 设置页面：渲染后端选择
- [x] 3.1 添加渲染后端下拉选择框 ✓ 已完成
- [x] 3.2 显示当前后端状态和可用性 ✓ 已完成
- [x] 3.3 保存用户选择到配置 ✓ 已完成
  - GuiConfig 结构体保存到 ~/.config/wayvid/gui.yaml
  - 后端、文件夹、语言设置自动持久化
- [x] 3.4 添加后端切换需要重启的提示 ✓ 已完成

## 4. GUI 壁纸库优化：自定义文件夹
- [x] 4.1 将"导入壁纸"改为"添加壁纸文件夹" ✓ 已完成
- [x] 4.2 实现文件夹递归扫描功能 ✓ 已完成
- [x] 4.3 添加文件夹路径持久化存储 ✓ 已完成
  - 文件夹列表保存到 gui.yaml，启动时自动恢复
- [x] 4.4 添加移除文件夹功能 ✓ 已完成
- [x] 4.5 支持文本输入添加文件夹 ✓ 已完成 (使用文本输入代替 rfd)

## 5. GUI Workshop 预览图支持
- [x] 5.1 在 WallpaperItem 中添加预览图路径字段 ✓ 已完成
- [x] 5.2 扫描时加载 preview.gif/preview.jpg 路径 ✓ 已完成
- [x] 5.3 实现预览图加载和缓存 ✓ 已完成 (preview_textures HashMap)
- [x] 5.4 在壁纸网格中显示预览缩略图 ✓ 已完成
- [x] 5.5 支持 GIF 动画预览 ✓ 已完成
  - GifAnimation 结构存储帧和延迟
  - show_gif_preview() 实现帧动画播放
  - 使用 request_repaint_after() 触发动画更新

## 6. GUI 场景壁纸支持
- [x] 6.1 扫描时检测壁纸类型（video/scene）✓ 已完成
- [x] 6.2 在 WallpaperItem 中添加壁纸类型字段 ✓ 已完成
- [x] 6.3 显示场景壁纸标签 ✓ 已完成 (紫色 "🎬 Scene" 标签)
- [x] 6.4 实现场景壁纸应用功能 ✓ 已完成
  - apply_wallpaper() 根据 WallpaperType 发送正确的 IPC 命令
  - Scene 类型使用 SwitchSource + VideoSource::WeScene
- [ ] 6.5 添加场景壁纸过滤选项 (可选，暂不实现)

## 7. 测试和文档
- [ ] 7.1 测试渲染后端切换
- [ ] 7.2 测试文件夹扫描功能
- [ ] 7.3 测试预览图显示
- [ ] 7.4 测试场景壁纸应用
- [ ] 7.5 更新用户文档
