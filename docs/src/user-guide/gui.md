# GUI

```bash
wayvid-gui
```

## Interface

```
┌─────────────────────────────────────────────┐
│  [Wallpapers]  [Settings]          [🌐 EN] │
├─────────────────────────────────────────────┤
│                                             │
│   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐         │
│   │ Vid │ │ Vid │ │ Vid │ │ Vid │  [+Add] │
│   └─────┘ └─────┘ └─────┘ └─────┘         │
│                                             │
├─────────────────────────────────────────────┤
│  [DP-1 ✓]  [HDMI-A-1]  [eDP-1]             │
└─────────────────────────────────────────────┘
```

## Usage

1. **Select monitor** - Click in bottom bar
2. **Choose wallpaper** - Browse grid
3. **Apply** - Double-click wallpaper

## Features

- Start/stop daemon
- Browse local videos and Workshop items
- Per-monitor configuration
- Language: English / 简体中文

## Keyboard

| Key | Action |
|-----|--------|
| Enter | Apply wallpaper |
| Space | Play/pause |
| Ctrl+R | Reload config |
| Ctrl+Q | Quit |

## Troubleshooting

**No monitors shown:**
- Check Wayland session
- Restart GUI

**Fonts broken (Chinese):**
```bash
sudo pacman -S noto-fonts-cjk  # Arch
```
