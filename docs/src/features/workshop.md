# Steam Workshop Integration

wayvid supports importing video wallpapers from Wallpaper Engine's Steam Workshop, making it easy to use thousands of community-created wallpapers.

## Quick Start

```bash
# Method 1: Use subscribed items from Steam
wayvid workshop list                    # See subscribed items
wayvid workshop import <id> -o config.yaml

# Method 2: Direct install by ID
wayvid workshop install <id> -o ~/.config/wayvid/config.yaml
```

## Three Methods to Use Workshop

### Method 1: Subscribe in Steam (Recommended)

1. Browse [Workshop](https://steamcommunity.com/app/431960/workshop/) and subscribe to video wallpapers
2. List subscribed items: `wayvid workshop list`
3. Import: `wayvid workshop import <id> -o config.yaml`

### Method 2: Direct Download

1. Find item ID from Workshop URL (`?id=XXXXXXXXX`)
2. Install directly: `wayvid workshop install <id> -o config.yaml`

### Method 3: Cached Items

Downloaded items are cached in `~/.cache/wayvid/workshop/` for reuse.

```bash
wayvid workshop cache                    # List cached
wayvid workshop cache --clear-item <id>  # Clear one
wayvid workshop cache --clear            # Clear all
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `list` | List Steam subscribed items | `wayvid workshop list` |
| `info <id>` | Show item details | `wayvid workshop info 1234567890` |
| `search <query>` | Get search instructions | `wayvid workshop search anime` |
| `download <id>` | Download item | `wayvid workshop download <id>` |
| `install <id>` | Download + import | `wayvid workshop install <id> -o config.yaml` |
| `import <id>` | Import local/cached item | `wayvid workshop import <id> -o config.yaml` |
| `cache` | Manage downloads | `wayvid workshop cache --clear` |

## Finding Workshop IDs

### From Workshop URL

```
https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890
                                                         ^^^^^^^^^^
                                                         This is the ID
```

### From Steam Client

1. Right-click wallpaper in Steam Workshop
2. "Copy Page URL"
3. Extract ID from URL

### Popular Collections

- [Trending Videos](https://steamcommunity.com/workshop/browse/?appid=431960&browsesort=trend&requiredtags%5B%5D=Video)
- [Most Subscribed](https://steamcommunity.com/workshop/browse/?appid=431960&browsesort=totaluniquesubscribers&requiredtags%5B%5D=Video)

## Compatibility

### ✅ Supported
- Video wallpapers (.mp4, .webm, .mkv)
- Image sequences
- Basic scene projects with video

### ❌ Not Supported  
- HTML wallpapers
- WebGL shaders
- Audio-reactive features
- Interactive elements

**Tip**: Look for "Video" tag in Workshop to ensure compatibility.

## Examples

### Single Monitor

```bash
# Install popular wallpaper
wayvid workshop install 2815866033 -o ~/.config/wayvid/config.yaml

# Start wayvid
wayvid
```

### Multi-Monitor

```bash
# Import for each monitor
wayvid workshop import 1111111111 > left.yaml
wayvid workshop import 2222222222 > right.yaml

# Manually merge with per_output in main config
# See: Multi-Monitor guide
```

## Troubleshooting

### "Steam installation not found"

**Solution**: Use direct download instead:
```bash
wayvid workshop download <id>
```

### "Direct download not available"

**Solution**: Subscribe in Steam client, then use:
```bash
wayvid workshop list
wayvid workshop import <id>
```

### "Invalid project" / "No video file"

**Cause**: Item is not a video wallpaper (HTML/WebGL).

**Solution**: Only subscribe to items tagged as "Video".

## See Also

- [Configuration Reference](../reference/config.md)
- [WE Format](../reference/we-format.md)
- [Multi-Monitor Setup](../user-guide/multi-monitor.md)
