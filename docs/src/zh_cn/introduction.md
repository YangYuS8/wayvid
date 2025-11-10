# wayvid

**Wayland 动态壁纸守护进程**

wayvid 是一个轻量、高性能的 Wayland 动态壁纸解决方案,专为现代 Linux 桌面环境设计。

## 核心特性

- ⚡ **零拷贝渲染**: 共享 GPU 解码缓冲区,内存占用极低
- 🎨 **HDR10 支持**: 原生 HDR 色彩管理和色调映射
- 🔄 **热重载配置**: 无需重启即可更改壁纸
- 🖥️ **多显示器**: 每个显示器独立视频源和布局
- 🎮 **Steam 创意工坊**: 一键导入 Wallpaper Engine 壁纸
- 🎵 **音频支持**: 可选背景音频播放
- 🔌 **IPC 控制**: 通过命令行或 GUI 实时控制

## 快速开始

```bash
# 安装 (Arch Linux)
yay -S wayvid

# 创建配置
mkdir -p ~/.config/wayvid
nano ~/.config/wayvid/config.yaml

# 启动守护进程
wayvid &

# 控制播放
wayvid-ctl play
```

详见 [快速开始指南](user-guide/quick-start.md)。

## 支持的合成器

- ✅ Hyprland
- ✅ Niri (原生优化)
- ✅ Sway
- ✅ River
- ⚠️ 其他 wlr-layer-shell 合成器 (部分支持)

## 系统要求

- **OS**: Linux (Wayland)
- **GPU**: OpenGL ES 3.0+ 或 Vulkan
- **解码**: VA-API / VDPAU / NVDEC 硬件解码 (推荐)
- **依赖**: mpv, ffmpeg, mesa

## 项目状态

- 🟢 **稳定版本**: v0.3.x
- 🔵 **开发分支**: main
- 📦 **打包**: AUR, Nix flake

## 文档导航

### 用户
- [安装指南](user-guide/installation.md)
- [配置说明](user-guide/configuration.md)
- [功能特性](features/hdr.md)

### 开发者
- [构建指南](dev/building.md)
- [系统架构](dev/architecture.md)
- [贡献指南](dev/contributing.md)

## 获取帮助

- 🐛 [问题反馈](https://github.com/your-username/wayvid/issues)
- 💬 [讨论区](https://github.com/your-username/wayvid/discussions)
- 📖 [完整文档](https://your-username.github.io/wayvid)

## 许可证

MIT License - 详见 [LICENSE-MIT](https://github.com/your-username/wayvid/blob/main/LICENSE-MIT)
