# Change: 修复 GUI 布局严重问题

## Why

当前 GUI 存在三个严重的布局和显示问题，严重影响用户体验：

1. **侧边栏宽度失控**：展开的侧边栏使用了 `Length::Fill` 而非固定宽度，导致侧边栏占据整个应用的一半页面，且随窗口缩放保持 50% 比例
2. **图标乱码**：Emoji 图标（📚📁🖥️⚙️ℹ️🎬🎨等）在某些系统上显示为带叉的方框，因为系统缺少 Emoji 字体支持
3. **缩略图比例变化**：壁纸缩略图使用 `Length::Fill` 宽度，当面板展开/收起时宽度变化导致图片裁切不一致

## What Changes

### 1. 侧边栏固定宽度（**关键修复**）
- 展开状态的侧边栏容器添加 `width(Length::Fixed(sidebar_width))`
- 确保侧边栏宽度始终为 180px（展开）/ 50px（折叠）

### 2. 图标字符替换
- 将所有 Emoji 图标替换为纯 ASCII 文字或 Unicode 符号
- 导航：📚→"库" / 📁→"夹" / 🖥️→"屏" / ⚙️→"设" / ℹ️→"关"
- 或使用简单 ASCII 图标：[L] [F] [M] [S] [?]

### 3. 缩略图固定尺寸
- 缩略图使用固定的 1:1 正方形比例
- 使用 `Length::Fixed(120.0)` 设置宽度和高度
- 使用 `ContentFit::Cover` 保持填充裁切

## Impact

- **Affected specs**: `gui-integration`
- **Affected code**: 
  - `crates/wayvid-gui/src/app.rs` - 侧边栏布局
  - `crates/wayvid-gui/src/views/library.rs` - 缩略图布局
