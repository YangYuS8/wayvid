# Wallpaper Engine Format Reference

Reference document for Wallpaper Engine project format parsing.

## Project Structure

Wallpaper Engine projects are typically organized as follows:

```
workshop_id/
├── project.json          # Main project metadata
├── preview.jpg           # Preview image (optional)
├── scene.pkg            # Scene package (for web/scene wallpapers)
├── video.mp4            # Video file (for video wallpapers)
└── audio.mp3            # Audio file (optional)
```

## project.json Format

### Video Wallpaper Example

```json
{
  "approved": true,
  "contentrating": "Everyone",
  "description": "A beautiful ocean scene",
  "file": "video.mp4",
  "general": {
    "properties": {
      "audioprocessing": {
        "value": false
      },
      "schemecolor": {
        "value": "0.2 0.4 0.6"
      }
    }
  },
  "preview": "preview.jpg",
  "tags": ["Nature", "Ocean"],
  "title": "Ocean Waves",
  "type": "video",
  "visibility": "public",
  "workshopid": "123456789",
  "workshopurl": "steam://url/CommunityFilePage/123456789"
}
```

### Key Fields for Video Wallpapers

| Field | Type | Description | wayvid Mapping |
|-------|------|-------------|----------------|
| `type` | string | Wallpaper type | Must be "video" for import |
| `file` | string | Video file path | `source.path` |
| `title` | string | Wallpaper title | Metadata only |
| `description` | string | Description | Metadata only |
| `preview` | string | Preview image | Ignored |
| `workshopid` | string | Steam Workshop ID | Metadata only |

### Video-Specific Properties

Properties are nested under `general.properties`:

```json
{
  "general": {
    "properties": {
      "rate": {
        "order": 0,
        "text": "ui_editor_properties_playback_rate",
        "type": "slider",
        "value": 1.0,
        "min": 0.1,
        "max": 5.0,
        "fraction": true
      },
      "volume": {
        "order": 1,
        "text": "ui_editor_properties_volume",
        "type": "slider",
        "value": 50.0,
        "min": 0.0,
        "max": 100.0,
        "fraction": false
      },
      "playbackmode": {
        "order": 2,
        "text": "ui_editor_properties_playback_mode",
        "type": "combo",
        "value": 0,
        "options": [
          {"label": "ui_editor_properties_playback_loop", "value": 0},
          {"label": "ui_editor_properties_playback_pause", "value": 1}
        ]
      },
      "audioprocessing": {
        "order": 3,
        "text": "ui_editor_properties_audioprocessing",
        "type": "bool",
        "value": false
      },
      "alignment": {
        "order": 4,
        "text": "ui_editor_properties_alignment",
        "type": "combo",
        "value": 0,
        "options": [
          {"label": "ui_editor_properties_alignment_center", "value": 0},
          {"label": "ui_editor_properties_alignment_fit", "value": 1},
          {"label": "ui_editor_properties_alignment_fill", "value": 2},
          {"label": "ui_editor_properties_alignment_stretch", "value": 3}
        ]
      },
      "schemecolor": {
        "order": 5,
        "text": "ui_editor_properties_scheme_color",
        "type": "color",
        "value": "0 0 0"
      }
    }
  }
}
```

## Property Mappings

### Playback Rate

**WE Property:** `general.properties.rate.value`
- Type: float
- Range: 0.1 - 5.0
- Default: 1.0

**wayvid Mapping:**
```yaml
playback_rate: 1.0  # Direct mapping
```

### Volume

**WE Property:** `general.properties.volume.value`
- Type: float
- Range: 0.0 - 100.0
- Default: 50.0

**wayvid Mapping:**
```yaml
volume: 50.0  # Direct mapping
mute: false   # Derived: mute if volume == 0.0
```

### Playback Mode (Loop)

**WE Property:** `general.properties.playbackmode.value`
- Type: integer enum
- Values:
  - 0: Loop (continuous playback)
  - 1: Pause (single play, then pause)

**wayvid Mapping:**
```yaml
loop: true   # playbackmode == 0
# playbackmode == 1: loop: false (not fully supported yet)
```

### Alignment (Layout)

**WE Property:** `general.properties.alignment.value`
- Type: integer enum
- Values:
  - 0: Center (video centered, no scaling)
  - 1: Fit (fit inside screen, preserve aspect)
  - 2: Fill (fill screen, crop if needed, preserve aspect)
  - 3: Stretch (stretch to fill, ignore aspect)

**wayvid Mapping:**
```yaml
layout: Centre   # alignment == 0
layout: Contain  # alignment == 1
layout: Cover    # alignment == 2
layout: Fill     # alignment == 3
```

### Audio Processing

**WE Property:** `general.properties.audioprocessing.value`
- Type: boolean
- Default: false

**wayvid Mapping:**
```yaml
# Not directly supported
# If true, WE applies audio reactivity (not in scope)
```

### Scheme Color

**WE Property:** `general.properties.schemecolor.value`
- Type: RGB color string (space-separated 0-1 values)
- Example: "0.2 0.4 0.6"

**wayvid Mapping:**
```yaml
# Metadata only, not used for video rendering
```

## Parsing Strategy

### 1. Project Detection

Check for `project.json` in the given directory:

```rust
use std::path::{Path, PathBuf};
use serde_json::Value;

pub fn detect_we_project(path: &Path) -> Result<PathBuf> {
    let project_file = path.join("project.json");
    if project_file.exists() {
        Ok(project_file)
    } else {
        Err(anyhow!("No project.json found in {}", path.display()))
    }
}
```

### 2. Type Validation

Ensure the project is a video wallpaper:

```rust
pub fn validate_project_type(json: &Value) -> Result<()> {
    let project_type = json["type"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing 'type' field"))?;
    
    if project_type != "video" {
        return Err(anyhow!(
            "Unsupported project type: '{}'. Only 'video' type is supported.",
            project_type
        ));
    }
    
    Ok(())
}
```

### 3. Video File Resolution

The `file` field is relative to the project directory:

```rust
pub fn resolve_video_path(project_dir: &Path, json: &Value) -> Result<PathBuf> {
    let file = json["file"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing 'file' field"))?;
    
    let video_path = project_dir.join(file);
    
    if !video_path.exists() {
        return Err(anyhow!("Video file not found: {}", video_path.display()));
    }
    
    Ok(video_path)
}
```

### 4. Property Extraction

Extract properties with defaults:

```rust
pub fn extract_properties(json: &Value) -> WeProperties {
    let props = &json["general"]["properties"];
    
    WeProperties {
        rate: props["rate"]["value"].as_f64().unwrap_or(1.0),
        volume: props["volume"]["value"].as_f64().unwrap_or(50.0),
        playback_mode: props["playbackmode"]["value"].as_i64().unwrap_or(0),
        alignment: props["alignment"]["value"].as_i64().unwrap_or(0),
        audio_processing: props["audioprocessing"]["value"].as_bool().unwrap_or(false),
    }
}
```

### 5. Config Generation

Convert to wayvid config:

```rust
pub fn generate_wayvid_config(
    video_path: PathBuf,
    props: WeProperties,
) -> Result<Config> {
    let layout = match props.alignment {
        0 => LayoutMode::Centre,
        1 => LayoutMode::Contain,
        2 => LayoutMode::Cover,
        3 => LayoutMode::Fill,
        _ => LayoutMode::Contain, // Default fallback
    };
    
    let mute = props.volume == 0.0;
    
    Ok(Config {
        source: VideoSource::File {
            path: video_path.to_string_lossy().to_string(),
        },
        layout,
        loop_playback: props.playback_mode == 0,
        start_time: 0.0,
        playback_rate: props.rate,
        mute,
        volume: props.volume,
        hwdec: true,
        per_output: HashMap::new(),
    })
}
```

## Error Handling

### Common Issues

1. **Missing project.json**
   - Error: "No project.json found"
   - Solution: Verify correct directory path

2. **Unsupported Type**
   - Error: "Unsupported project type: 'web'"
   - Solution: Only video wallpapers are supported

3. **Missing Video File**
   - Error: "Video file not found: path/to/video.mp4"
   - Solution: Ensure all files are present in the project directory

4. **Invalid JSON**
   - Error: "Failed to parse project.json"
   - Solution: Validate JSON syntax

5. **Missing Required Fields**
   - Warning: Use defaults for missing optional properties
   - Error: Fail only if critical fields (type, file) are missing

## Example Conversion

### Input: WE project.json

```json
{
  "type": "video",
  "file": "ocean.mp4",
  "title": "Ocean Waves",
  "general": {
    "properties": {
      "rate": {"value": 1.2},
      "volume": {"value": 30.0},
      "playbackmode": {"value": 0},
      "alignment": {"value": 2}
    }
  }
}
```

### Output: wayvid config.yaml

```yaml
source:
  type: File
  path: /path/to/workshop/123456789/ocean.mp4

layout: Cover
loop: true
start_time: 0.0
playback_rate: 1.2
mute: false
volume: 30.0
hwdec: true

# Imported from Wallpaper Engine
# Title: Ocean Waves
# Workshop ID: 123456789
```

## Implementation Checklist

- [ ] Add `serde_json` dependency
- [ ] Create `src/we/` module:
  - [ ] `mod.rs` - Module exports
  - [ ] `parser.rs` - JSON parsing logic
  - [ ] `types.rs` - WE-specific types
  - [ ] `converter.rs` - WE → wayvid conversion
- [ ] Implement `wayvid import` CLI command
- [ ] Add tests with sample project.json files
- [ ] Update README with import guide
- [ ] Create example WE project for testing

## References

- Wallpaper Engine Steam Workshop: https://steamcommunity.com/workshop/browse/?appid=431960
- WE project format: Reverse-engineered from actual projects
- Common locations:
  - Windows: `C:\Program Files (x86)\Steam\steamapps\workshop\content\431960\`
  - Linux (Proton): `~/.steam/steam/steamapps/workshop/content/431960/`

---

**Last Updated:** October 23, 2025  
**Version:** M4 Phase 1
