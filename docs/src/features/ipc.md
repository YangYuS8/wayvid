# IPC Control

Control wayvid in real-time via CLI or programmatically.

## CLI Tool

`wayvid-ctl` provides IPC commands:

### Status
```bash
wayvid-ctl status
```

### Playback Control
```bash
wayvid-ctl play [--output DP-1]
wayvid-ctl pause [--output DP-1]
wayvid-ctl stop [--output DP-1]
wayvid-ctl resume [--output DP-1]
```

### Configuration
```bash
wayvid-ctl reload-config
wayvid-ctl set-volume 50
wayvid-ctl set-source /path/to/video.mp4
```

### Output Management
```bash
wayvid-ctl list-outputs
wayvid-ctl enable-output DP-1
wayvid-ctl disable-output HDMI-A-1
```

## IPC Protocol

Unix socket: `$XDG_RUNTIME_DIR/wayvid.sock`

### Message Format

JSON-based protocol:

**Request:**
```json
{
  "command": "play",
  "params": {
    "output": "DP-1"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {}
}
```

### Available Commands

| Command | Parameters | Description |
|---------|------------|-------------|
| `status` | - | Get current status |
| `play` | `output?` | Start playback |
| `pause` | `output?` | Pause playback |
| `stop` | `output?` | Stop playback |
| `resume` | `output?` | Resume playback |
| `reload_config` | - | Reload config file |
| `set_volume` | `volume: u8` | Set volume (0-100) |
| `set_source` | `path: string` | Change video source |
| `list_outputs` | - | List all outputs |

## Programming Examples

### Python
```python
import socket
import json

sock = socket.socket(socket.AF_UNIX)
sock.connect('/run/user/1000/wayvid.sock')

# Send command
cmd = {"command": "pause", "params": {}}
sock.send(json.dumps(cmd).encode())

# Receive response
resp = json.loads(sock.recv(1024))
print(resp)
```

### Bash
```bash
echo '{"command":"status","params":{}}' | nc -U $XDG_RUNTIME_DIR/wayvid.sock
```

### Rust
```rust
use wayvid::ctl::protocol::{IpcCommand, IpcResponse};

let cmd = IpcCommand::Pause { output: None };
let resp: IpcResponse = send_ipc_command(cmd)?;
```

## GUI Integration

Desktop GUI available:

```bash
wayvid-gui
```

Features:
- Output status monitoring
- Playback controls
- Configuration editing
- Workshop browser

## Troubleshooting

**Socket not found:**
- Ensure wayvid is running
- Check: `ls $XDG_RUNTIME_DIR/wayvid.sock`

**Permission denied:**
- Socket owned by wayvid user
- Run wayvid-ctl as same user

**No response:**
- Check wayvid logs
- Verify JSON syntax
- Use `--log-level debug`

## See Also

- [Configuration Reference](../reference/config.md)
- [IPC Protocol Reference](../reference/ipc-protocol.md)
