# IPC Protocol

wayvid's IPC protocol specification.

## Transport

**Protocol**: JSON over Unix socket  
**Socket**: `$XDG_RUNTIME_DIR/wayvid.sock`  
**Format**: Newline-delimited JSON

## Message Format

### Request
```json
{
  "command": "string",
  "params": {}
}
```

### Response
```json
{
  "success": boolean,
  "data": {},
  "error": "string?"
}
```

## Commands

### status
Get daemon status.

**Request:**
```json
{"command": "status", "params": {}}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "running": true,
    "outputs": [
      {
        "name": "DP-1",
        "resolution": "2560x1440",
        "playing": true
      }
    ]
  }
}
```

### play
Start playback.

**Request:**
```json
{
  "command": "play",
  "params": {
    "output": "DP-1"  // optional
  }
}
```

### pause
Pause playback.

**Request:**
```json
{
  "command": "pause",
  "params": {
    "output": "DP-1"  // optional
  }
}
```

### stop
Stop playback.

**Request:**
```json
{
  "command": "stop",
  "params": {
    "output": "DP-1"  // optional
  }
}
```

### resume
Resume playback.

**Request:**
```json
{
  "command": "resume",
  "params": {
    "output": "DP-1"  // optional
  }
}
```

### reload_config
Reload configuration file.

**Request:**
```json
{"command": "reload_config", "params": {}}
```

### set_volume
Set volume.

**Request:**
```json
{
  "command": "set_volume",
  "params": {
    "volume": 50  // 0-100
  }
}
```

### set_source
Change video source.

**Request:**
```json
{
  "command": "set_source",
  "params": {
    "path": "/path/to/video.mp4"
  }
}
```

### list_outputs
List all outputs.

**Request:**
```json
{"command": "list_outputs", "params": {}}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "outputs": ["DP-1", "HDMI-A-1"]
  }
}
```

## Error Handling

### Error Response
```json
{
  "success": false,
  "error": "Output 'DP-1' not found"
}
```

### Common Errors
- `"daemon not running"`
- `"invalid command"`
- `"output not found"`
- `"invalid parameters"`

## Example Implementation

See [IPC Control](../features/ipc.md#programming-examples) for code examples.

## Versioning

Protocol version in response:
```json
{
  "success": true,
  "version": "1.0",
  "data": {}
}
```

Breaking changes will increment major version.
