# Wallpaper Engine Format

Specification for Wallpaper Engine project files.

## Project Structure

```
workshop_id/
├── project.json       # Manifest
├── scene.json         # Scene definition (optional)
├── preview.jpg        # Thumbnail
└── assets/            # Media files
    ├── video.mp4
    ├── audio.mp3
    └── ...
```

## project.json

Main manifest file.

### Required Fields

```json
{
  "title": "My Wallpaper",
  "type": "video",
  "file": "assets/video.mp4"
}
```

### Full Format

```json
{
  "title": "Wallpaper Title",
  "description": "Description",
  "type": "video",
  "file": "assets/video.mp4",
  "preview": "preview.jpg",
  "tags": ["anime", "scenery"],
  "general": {
    "properties": {
      "schemecolor": "0 0 0"
    }
  }
}
```

## Supported Types

wayvid supports these types:

- `video`: Video file (MP4, WebM)
- `web`: HTML/CSS/JS (limited support)
- `scene`: Scene-based (converted to video)

## File References

File paths are relative to project root:

```json
{
  "file": "assets/video.mp4",      // ✅ Relative
  "file": "/absolute/path.mp4",     // ❌ Not supported
  "file": "../outside.mp4"          // ❌ Not supported
}
```

## Audio Handling

wayvid extracts audio automatically:

```json
{
  "type": "video",
  "file": "video_with_audio.mp4"  // Audio track used if present
}
```

## Scene Format (Limited)

Basic scene.json support:

```json
{
  "objects": [
    {
      "image": "assets/bg.jpg",
      "origin": "0 0",
      "scale": "1 1"
    }
  ]
}
```

**Note**: Complex scenes may not render correctly.

## Conversion Process

When importing:

1. Parse `project.json`
2. Locate video file
3. Verify file exists
4. Extract metadata
5. Copy to wayvid cache: `~/.cache/wayvid/workshop/<id>/`

## Workshop ID Format

Steam Workshop IDs are numeric:
```
https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890
                                                          ^^^^^^^^^^
                                                          Workshop ID
```

## Validation Rules

wayvid validates:
- ✅ `project.json` must exist
- ✅ `file` field must be present
- ✅ Referenced file must exist
- ✅ File must be valid video format
- ⚠️ Audio is optional
- ⚠️ `preview.jpg` is optional

## Error Handling

### Common Issues

**Missing file:**
```json
{
  "file": "video.mp4"  // File doesn't exist
}
```
→ Error: "File not found: video.mp4"

**Invalid type:**
```json
{
  "type": "unknown"
}
```
→ Fallback to video detection

**Missing project.json:**
→ Error: "Invalid Workshop item"

## Examples

### Simple Video

```json
{
  "title": "Ocean Waves",
  "type": "video",
  "file": "ocean.mp4"
}
```

### Video with Audio

```json
{
  "title": "Rainy City",
  "type": "video",
  "file": "city_rain.mp4",
  "tags": ["rain", "city", "relaxing"]
}
```

### Web-Based (Experimental)

```json
{
  "title": "Clock Widget",
  "type": "web",
  "file": "index.html"
}
```

**Note**: Web support is experimental. Complex WebGL/Canvas may not work.

## Reference Implementation

See `src/we/` for parsing code:
- `src/we/parser.rs` - JSON parsing
- `src/we/converter.rs` - Format conversion
- `src/we/workshop.rs` - Steam integration

## Related

- [Workshop Integration](../features/workshop.md)
- [Configuration Reference](config.md)
