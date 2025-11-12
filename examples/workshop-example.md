# Workshop Integration Example

This example demonstrates how to use wayvid's Steam Workshop integration.

## Quick Example

```bash
# 1. Install a Workshop wallpaper by ID
wayvid workshop install 2815866033 -o ~/.config/wayvid/config.yaml

# 2. Run wayvid
wayvid
```

## Finding Workshop IDs

Visit the [Steam Workshop](https://steamcommunity.com/app/431960/workshop/) and find wallpapers you like.

The ID is in the URL:
```
https://steamcommunity.com/sharedfiles/filedetails/?id=2815866033
                                                         ^^^^^^^^^^
                                                         Workshop ID
```

## Popular Video Wallpapers

Here are some popular video wallpapers to try (IDs may change):

- **Rainy Night City**: Search for "rainy night" in Workshop
- **Sakura Garden**: Search for "sakura" in Workshop
- **Space Scene**: Search for "space" with "Video" tag

## Complete Workflow

### Using Steam Client (Recommended)

```bash
# 1. Subscribe to wallpapers in Steam
# 2. List subscribed items
wayvid workshop list

# Example output:
# 📦 Found 3 Workshop items:
#
#   [1234567890] Animated Sakura
#       📁 ~/.local/share/Steam/steamapps/workshop/content/431960/1234567890/scene.mp4
#
#   [9876543210] Neon City
#       📁 ~/.local/share/Steam/steamapps/workshop/content/431960/9876543210/video.webm

# 3. Import to config
wayvid workshop import 1234567890 -o ~/.config/wayvid/config.yaml

# 4. Start wayvid
wayvid
```

### Direct Download

```bash
# 1. Find Workshop ID from URL
# 2. Download and install in one command
wayvid workshop install 2815866033 -o ~/.config/wayvid/config.yaml

# 3. Start wayvid
wayvid
```

## Multi-Monitor Setup

```bash
# Import wallpapers for each monitor
wayvid workshop import 1111111111 > monitor1.yaml
wayvid workshop import 2222222222 > monitor2.yaml

# Then manually edit config.yaml to use per_output:
# outputs:
#   eDP-1:
#     source:
#       type: file
#       path: ~/.local/share/Steam/.../1111111111/scene.mp4
#   HDMI-A-1:
#     source:
#       type: file
#       path: ~/.local/share/Steam/.../2222222222/video.webm
```

## Cache Management

```bash
# List cached downloads
wayvid workshop cache

# Clear specific item
wayvid workshop cache --clear-item 1234567890

# Clear all cached downloads
wayvid workshop cache --clear
```

## Troubleshooting

### No Steam Installation

If you don't have Steam installed, you can still use direct download:

```bash
wayvid workshop download 2815866033
wayvid workshop import 2815866033 -o config.yaml
```

### Item Not Downloading

Some Workshop items require Steam client authentication. In this case:

1. Install Steam
2. Subscribe to the item in Steam Workshop
3. Use `wayvid workshop list` to see it
4. Import with `wayvid workshop import <id>`

### Wrong Wallpaper Type

Only **video wallpapers** work with wayvid. Check:

- Look for "Video" tag in Workshop
- Avoid "Web", "Scene", or "Application" types
- Check file list for `.mp4`, `.webm`, or `.mkv` files

## See Also

- [Workshop Documentation](../docs/src/features/workshop.md)
- [Configuration Reference](../docs/src/reference/config.md)
- [Multi-Monitor Setup](../docs/src/user-guide/multi-monitor.md)
