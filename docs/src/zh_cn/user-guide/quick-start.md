# 快速开始

5分钟启动 wayvid。

## 1. 安装

### Arch Linux
```bash
yay -S wayvid
```

### 其他发行版
详见[安装指南](installation.md)。

## 2. 创建配置

```bash
mkdir -p ~/.config/wayvid
```

最小配置 `~/.config/wayvid/config.yaml`:
```yaml
outputs:
  default:
    source:
      type: file
      path: ~/Videos/wallpaper.mp4
    layout: fill
    volume: 0
```

示例配置:
```bash
cp /usr/share/wayvid/config.example.yaml ~/.config/wayvid/config.yaml
```

## 3. 准备视频

支持的格式: MP4, WebM, MKV, MOV

```bash
# 下载示例视频
mkdir -p ~/Videos
wget https://example.com/sample.mp4 -O ~/Videos/wallpaper.mp4
```

或使用 Steam 创意工坊:
```bash
wayvid workshop import 1234567890
```

## 4. 启动守护进程

### 手动启动
```bash
wayvid &
```

### 自动启动 (Hyprland)
`~/.config/hypr/hyprland.conf`:
```
exec-once = wayvid
```

### 自动启动 (systemd)
```bash
systemctl --user enable --now wayvid.service
```

## 5. 控制播放

```bash
# 播放
wayvid-ctl play

# 暂停
wayvid-ctl pause

# 停止
wayvid-ctl stop

# 调整音量
wayvid-ctl set-volume 30

# 查看状态
wayvid-ctl status
```

## 常见问题

### 黑屏无内容
检查日志:
```bash
journalctl --user -u wayvid -f
```

常见原因:
- 视频文件路径错误
- 不支持的视频格式
- 缺少硬件解码支持

### 性能问题
启用硬件解码:
```yaml
outputs:
  default:
    hwdec: auto  # 或 vaapi / nvdec
```

### 多显示器配置
```yaml
outputs:
  DP-1:
    source:
      type: file
      path: ~/Videos/left.mp4
  HDMI-A-1:
    source:
      type: file
      path: ~/Videos/right.mp4
```

## 下一步

- [完整配置选项](configuration.md)
- [视频源类型](video-sources.md)
- [多显示器设置](multi-monitor.md)
- [HDR 支持](../features/hdr.md)
