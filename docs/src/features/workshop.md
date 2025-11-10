# Steam Workshop Integration

Import Wallpaper Engine projects from Steam Workshop.

## Prerequisites

- Steam installed
- Wallpaper Engine owned (not required, but helps)
- Workshop items subscribed

## Commands

### List Available Items
```bash
wayvid workshop list
```

### Get Item Info
```bash
wayvid workshop info <workshop_id>
```

### Import Item
```bash
wayvid workshop import <workshop_id>
```

Generates config entry:
```yaml
source:
  type: workshop
  id: 1234567890
```

## Finding Workshop IDs

### Method 1: Steam Workshop Page
URL format: `https://steamcommunity.com/sharedfiles/filedetails/?id=<ID>`

Example: `id=1234567890`

### Method 2: wayvid CLI
```bash
wayvid workshop list
```

Output:
```
Found 5 Workshop items:
  [1234567890] Awesome Wallpaper
  [9876543210] Cool Animation
  ...
```

## Supported Features

✅ Video wallpapers
✅ Static images
✅ Basic animations
⚠️ Interactive wallpapers (limited)
❌ Audio (muted by default)
❌ Web-based wallpapers

## Project Structure

Workshop items location:
```
~/.steam/steam/steamapps/workshop/content/431960/<id>/
```

wayvid automatically:
- Finds Steam installation
- Locates workshop items
- Parses `project.json`
- Converts to native format

## Configuration

```yaml
source:
  type: workshop
  id: 1234567890
  
  # Optional overrides
  volume: 0
  loop: true
```

## Troubleshooting

**Items not found:**
- Verify Steam is installed
- Check subscriptions in Steam Workshop
- Run `wayvid workshop list` to debug

**Playback issues:**
- Some WE features unsupported
- Check logs: `wayvid --log-level debug`
- Report issues on GitHub

## Performance

Workshop items are converted on-the-fly:
- First load: ~1-2s conversion
- Subsequent loads: instant

Cache location: `~/.cache/wayvid/workshop/`
