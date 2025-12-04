# Change: Enhance GUI and Rendering Backend Options

## Why

当前 wayvid GUI 存在以下问题需要改进：

1. **渲染后端选择**：用户无法在 GUI 中选择 OpenGL 或 Vulkan 后端
2. **壁纸管理体验差**：导入壁纸操作繁琐，需要逐个添加文件
3. **预览图缺失**：Steam Workshop 壁纸没有显示预览图（preview.gif/preview.jpg）
4. **场景壁纸不支持**：GUI 没有对场景类壁纸的支持

## What Changes

### Phase 1: 渲染后端选择
- 在设置页面添加渲染后端选择（OpenGL / Vulkan / Auto）
- 检测 Vulkan 可用性并给出提示
- 保存用户选择到配置文件

### Phase 2: 壁纸库管理优化
- 将"导入壁纸"改为"添加自定义壁纸文件夹"
- 支持递归扫描文件夹中的视频文件
- 记住用户添加的文件夹路径

### Phase 3: Workshop 预览图支持
- 加载并显示 Workshop 壁纸的 preview.gif 或 preview.jpg
- 在壁纸网格中显示预览缩略图
- 支持 GIF 动画预览

### Phase 4: 场景壁纸 GUI 支持
- 检测场景类型壁纸（project.json 中 type: "scene"）
- 显示场景壁纸的预览图
- 提供场景壁纸的应用功能
- 显示"场景壁纸"标签区分类型

## Impact

### 代码变更
- 修改 `src/bin/wayvid-gui.rs`：添加设置选项、预览图显示、场景支持
- 修改 `src/config/types.rs`：添加渲染后端配置字段
- 可能需要修改 `src/we/types.rs`：添加预览图路径字段

### 依赖变更
- 可能需要 `image` crate 支持 GIF 解码（已有依赖）

### 兼容性
- **非破坏性变更**：所有新功能为增量添加
- 保持向后兼容

### 风险
- GIF 动画可能影响 GUI 性能
- Vulkan 检测在某些系统上可能不准确

## Success Criteria
- 用户可以在设置中选择渲染后端
- 用户可以添加自定义壁纸文件夹并自动扫描
- Workshop 壁纸显示预览缩略图
- 场景类壁纸可以在 GUI 中识别和应用
