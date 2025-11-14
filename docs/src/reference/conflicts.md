# 与其他壁纸管理器的冲突

## 问题描述

wayvid 使用 Wayland Layer Shell 协议的 `Background` 层来显示壁纸。如果您的系统中同时运行了其他壁纸管理器（如 swww、hyprpaper、swaybg 等），它们可能会覆盖 wayvid 的输出，导致视频壁纸不可见。

## 常见冲突的壁纸管理器

### swww

**症状**：启动 wayvid 后桌面仍显示静态壁纸

**检测**：
```bash
ps aux | grep swww-daemon
```

**解决方案**：
```bash
# 临时停止
killall swww-daemon

# 永久禁用：从 compositor 配置中移除 swww 相关命令
# 例如 Niri: ~/.config/niri/config.kdl
# 注释或删除以下行：
# spawn-sh-at-startup "swww-daemon"
# spawn-sh-at-startup "swww img /path/to/image.png"
```

### hyprpaper (Hyprland)

**检测**：
```bash
ps aux | grep hyprpaper
```

**解决方案**：
```bash
# 停止 hyprpaper
killall hyprpaper

# 从 ~/.config/hypr/hyprland.conf 移除：
# exec-once = hyprpaper
```

### swaybg (Sway/SwayFX)

**检测**：
```bash
ps aux | grep swaybg
```

**解决方案**：
```bash
# 停止 swaybg
killall swaybg

# 从 Sway 配置移除：
# exec swaybg -i /path/to/image.png
```

## 自动检测

wayvid 会在启动时自动检测这些冲突的程序，并在日志中显示警告：

```
⚠️  Detected swww-daemon running
⚠️  swww and wayvid both use the Background layer
⚠️  This may cause wayvid to be hidden behind swww
⚠️  To fix: run 'killall swww-daemon' before starting wayvid
```

## 推荐配置流程

1. **停止现有壁纸管理器**：
   ```bash
   killall swww-daemon hyprpaper swaybg 2>/dev/null
   ```

2. **从 compositor 配置中移除它们的自动启动**

3. **将 wayvid 添加到自动启动**：
   ```bash
   # Niri 示例
   spawn-at-startup "wayvid" "run"
   
   # Hyprland 示例
   exec-once = wayvid run
   
   # Sway 示例
   exec wayvid run
   ```

4. **重启 compositor 或手动启动 wayvid**

## 验证

启动后检查进程：
```bash
ps aux | grep -E "(wayvid|swww|hyprpaper|swaybg)" | grep -v grep
```

应该只看到 `wayvid` 在运行。

## 技术细节

所有这些壁纸管理器都使用 Wayland Layer Shell 协议的 `Background` 层。该层是所有窗口之下的最底层，多个程序无法在同一层上共存。最后启动的程序通常会覆盖之前的程序。

未来版本可能会：
- 提供配置选项选择不同的 layer（如 `Bottom`）
- 自动检测并尝试终止冲突的进程
- 与 compositor 更深度集成
