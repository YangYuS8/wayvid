# wayvid - Quick Reference Card

## 命令速查

```bash
# 系统能力检查
wayvid check

# 运行壁纸引擎
wayvid run
wayvid run --config ~/.config/wayvid/config.yaml
wayvid run --log-level debug

# 构建
cargo build --release

# 安装
sudo install -Dm755 target/release/wayvid /usr/local/bin/

# 测试
cargo test --all-features
cargo clippy --all-features
cargo fmt --all
```

## 配置速查

```yaml
# 最小配置
source:
  type: File
  path: "/path/to/video.mp4"

# 完整配置
source:
  type: File
  path: "/path/to/video.mp4"
layout: Fill              # Fill | Contain | Stretch | Cover | Centre
loop: true
start_time: 0.0
playback_rate: 1.0
mute: true
volume: 0.0
hwdec: true

power:
  pause_when_hidden: true
  pause_on_battery: false
  max_fps: 30

per_output:
  HDMI-A-1:
    layout: Contain
  eDP-1:
    source:
      type: File
      path: "/path/to/other.mp4"
```

## 自启动速查

```bash
# Hyprland
echo 'exec-once = wayvid run' >> ~/.config/hypr/hyprland.conf

# niri
echo 'spawn-at-startup "wayvid" "run"' >> ~/.config/niri/config.kdl

# systemd
cp systemd/wayvid.service ~/.config/systemd/user/
systemctl --user enable --now wayvid.service
```

## 布局模式速查

| 模式 | 行为 | 用途 |
|------|------|------|
| **Fill** | 缩放并裁剪填满屏幕 | 推荐,无黑边 |
| **Contain** | 缩放以完整显示,加黑边 | 保持完整画面 |
| **Stretch** | 拉伸填满,忽略比例 | 特殊需求 |
| **Cover** | 同 Fill | 兼容别名 |
| **Centre** | 原始尺寸居中 | 小视频 |

## 故障排查速查

```bash
# 1. 检查 Wayland
echo $WAYLAND_DISPLAY              # 应有输出
echo $XDG_CURRENT_DESKTOP          # Hyprland/niri/等

# 2. 检查视频文件
mpv /path/to/video.mp4             # 应能播放

# 3. 检查硬解
vainfo                             # 应显示驱动信息

# 4. 查看日志
wayvid run --log-level debug

# 5. 检查输出名称
wayvid check | grep -i output

# 6. 测试 mpv
mpv --hwdec=auto /path/to/video.mp4
```

## 依赖安装速查

```bash
# Arch Linux
sudo pacman -S rust wayland libmpv mesa intel-media-driver

# Ubuntu/Debian
sudo apt install rustc cargo libwayland-dev libmpv-dev \
  libgl1-mesa-dev intel-media-va-driver

# Fedora
sudo dnf install rust cargo wayland-devel mpv-libs-devel \
  mesa-libGL-devel mesa-va-drivers

# NixOS
nix develop  # 使用 flake.nix
```

## 性能优化速查

```yaml
# 限制 FPS
power:
  max_fps: 30

# 禁用硬解(如有问题)
hwdec: false

# 降低视频分辨率
# 使用 1080p 而非 4K

# 电池模式暂停
power:
  pause_on_battery: true
```

## 文件位置速查

```
配置文件:     ~/.config/wayvid/config.yaml
二进制:       ~/.local/bin/wayvid 或 /usr/local/bin/wayvid
Service:     ~/.config/systemd/user/wayvid.service
日志:         journalctl --user -u wayvid -f
示例配置:     /usr/share/doc/wayvid/config.example.yaml
```

## 调试命令速查

```bash
# 查看 Wayland 协议
wayland-info | grep layer_shell

# 查看输出信息
wayland-info | grep wl_output

# 查看 GPU 信息
glxinfo | grep OpenGL
vainfo

# 查看进程
ps aux | grep wayvid

# 查看日志
journalctl --user -u wayvid -f

# 强制杀死
pkill -9 wayvid
```

## 常用 mpv 测试

```bash
# 测试硬解
mpv --hwdec=auto --log-file=mpv.log video.mp4
grep -i hwdec mpv.log

# 测试循环
mpv --loop=inf video.mp4

# 测试起始时间
mpv --start=10 video.mp4

# 测试速率
mpv --speed=1.5 video.mp4

# 查看属性
mpv --msg-level=all=v video.mp4
```

## Feature Flags 速查

```bash
# 默认特性
cargo build --release

# 所有特性
cargo build --release --all-features

# 自定义特性
cargo build --release --features "video-mpv,backend-wayland"

# 禁用默认特性
cargo build --release --no-default-features --features "video-mpv"
```

## Git 工作流速查

```bash
# 克隆
git clone https://github.com/yourusername/wayvid.git
cd wayvid

# 开发分支
git checkout -b feature/my-feature

# 提交
cargo fmt --all
cargo clippy --all-features
cargo test
git add .
git commit -m "描述"

# 推送
git push origin feature/my-feature
```

## 合成器兼容性速查

| 合成器 | Layer Shell | 状态 |
|--------|-------------|------|
| Hyprland | ✅ | ✅ 主要支持 |
| niri | ✅ | ✅ 主要支持 |
| Sway | ✅ | 🟡 应该工作 |
| River | ✅ | 🟡 应该工作 |
| KDE | ❌ | ❌ 不支持 |
| GNOME | ❌ | ❌ 不支持 |

## 环境变量速查

```bash
# 强制 VA-API 驱动
export LIBVA_DRIVER_NAME=iHD        # Intel (新)
export LIBVA_DRIVER_NAME=i965       # Intel (旧)
export LIBVA_DRIVER_NAME=radeonsi   # AMD
export LIBVA_DRIVER_NAME=nvidia     # NVIDIA

# Wayland
export WAYLAND_DISPLAY=wayland-1

# 日志级别
export RUST_LOG=wayvid=debug
```

## 视频格式速查

| 格式 | 编码 | 支持 | 备注 |
|------|------|------|------|
| MP4 | H.264 | ✅ | 推荐,通用 |
| MP4 | H.265 | ✅ | 更高效 |
| WebM | VP9 | ✅ | 开源 |
| WebM | AV1 | ✅ | 最新,高效 |
| MKV | 各种 | ✅ | 容器格式 |
| AVI | 各种 | 🟡 | 旧格式 |
| MOV | 各种 | 🟡 | Apple 格式 |

## 错误码速查

| 错误 | 原因 | 解决 |
|------|------|------|
| "Failed to connect to Wayland" | 不在 Wayland | 检查 $WAYLAND_DISPLAY |
| "zwlr_layer_shell_v1 not available" | 不支持的合成器 | 换用 Hyprland/niri/Sway |
| "Failed to create MPV instance" | libmpv 未安装 | 安装 mpv-libs |
| "Failed to load video file" | 文件不存在/损坏 | 检查路径和文件 |
| "Failed to get EGL display" | OpenGL 问题 | 检查显卡驱动 |

## 快速链接

- 📖 完整文档: [README.md](README.md)
- 🚀 快速开始: [QUICKSTART.md](QUICKSTART.md)
- 🛠️ 贡献指南: [CONTRIBUTING.md](CONTRIBUTING.md)
- 🏗️ 项目结构: [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)
- 📝 开发笔记: [DEV_NOTES.md](DEV_NOTES.md)
- 📊 M1 报告: [M1_DELIVERY_REPORT.md](M1_DELIVERY_REPORT.md)

## 社区资源

```
Issues:      https://github.com/yourusername/wayvid/issues
Discussions: https://github.com/yourusername/wayvid/discussions
```

---

**提示**: 保存此文件到 ~/Documents/wayvid-cheatsheet.md 以便快速查阅!
