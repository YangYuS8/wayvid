# Steam Workshop

Import video wallpapers from Wallpaper Engine.

## Using GUI (Recommended)

The easiest way to use Workshop wallpapers is through the GUI:

1. Open `wayvid-gui`
2. Go to **Folders** tab
3. Add your Workshop content folder:
   ```
   ~/.steam/steam/steamapps/workshop/content/431960/
   ```
4. Browse Workshop wallpapers in **Library** tab
5. Double-click to apply

## Using CLI

Apply Workshop wallpapers directly with wayvid-ctl:

```bash
# Find workshop items
ls ~/.steam/steam/steamapps/workshop/content/431960/

# Apply a workshop wallpaper
wayvid-ctl apply ~/.steam/steam/steamapps/workshop/content/431960/<id>/video.mp4
```

## Find Workshop ID

From URL:
```
https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890
                                                        ^^^^^^^^^^
```

The workshop content is typically at:
```
~/.steam/steam/steamapps/workshop/content/431960/<workshop_id>/
```

## Compatibility

**Supported:**
- Video wallpapers (.mp4, .webm, .mkv)

**Not supported:**
- Web/HTML wallpapers
- Scene wallpapers with effects
- Interactive wallpapers

Look for "Video" tag in Workshop.

## Troubleshooting

**"No video file found":**
- The wallpaper may not be a video type
- Check the folder for actual video files: `ls ~/.steam/.../431960/<id>/`

**Workshop folder not found:**
- Ensure Steam and Wallpaper Engine are installed
- Workshop content downloads to: `~/.steam/steam/steamapps/workshop/content/431960/`
