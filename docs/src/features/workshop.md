# Steam Workshop

Import video wallpapers from Wallpaper Engine.

## Quick Start

```bash
wayvid workshop list          # List subscribed items
wayvid workshop import <id>   # Import to config
```

## Commands

| Command | Description |
|---------|-------------|
| `list` | List subscribed items |
| `info <id>` | Show item details |
| `import <id>` | Import to config |
| `download <id>` | Download item |

## Find Workshop ID

From URL:
```
https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890
                                                        ^^^^^^^^^^
```

## Compatibility

**Supported:**
- Video wallpapers (.mp4, .webm, .mkv)

**Not supported:**
- Web/HTML wallpapers
- Scene wallpapers with effects
- Interactive wallpapers

Look for "Video" tag in Workshop.

## Example

```bash
# Import wallpaper
wayvid workshop import 2815866033 -o ~/.config/wayvid/config.yaml

# Start
systemctl --user restart wayvid.service
```

## Troubleshooting

**"Steam not found":**
- Use `wayvid workshop download <id>` instead

**"No video file":**
- Item is not a video wallpaper (try different one)
