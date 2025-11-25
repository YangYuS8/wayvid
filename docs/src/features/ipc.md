# IPC Control

Control wayvid in real-time via CLI, GUI, or programmatically.

## GUI Control Panel

The easiest way to control wayvid:

```bash
wayvid-gui
```

**Features:**
- 🖥️ **Multi-monitor display** with real-time status
- 🎬 **Playback controls** (play, pause, seek, volume)
- 📁 **Video source browser** with file picker
- 🎮 **Steam Workshop integration** with one-click apply
- ⚙️ **Configuration editor** with live preview
- 🌍 **Multi-language support** (English, 中文)

The GUI automatically connects to the daemon when it's running and shows connection status in real-time.

## CLI Tool

`wayvid-ctl` provides command-line IPC control:

### Status
```bash
# Get JSON status of all outputs
wayvid-ctl status
```

Output:
```json
{
  "version": "0.4.4-alpha.2",
  "outputs": [
    {
      "name": "HDMI-A-1",
      "width": 2560,
      "height": 1440,
      "playing": true,
      "paused": false
    }
  ]
}
```

### Playback Control
```bash
wayvid-ctl pause [--output HDMI-A-1]
wayvid-ctl resume [--output HDMI-A-1]
wayvid-ctl seek --output HDMI-A-1 --time 30.5
```

### Volume & Audio
```bash
wayvid-ctl volume --output HDMI-A-1 --volume 0.8
wayvid-ctl mute --output HDMI-A-1
```

### Layout
```bash
wayvid-ctl layout --output HDMI-A-1 --mode fill
# Modes: fill, contain, stretch, cover, centre
```

### Playback Rate
```bash
wayvid-ctl rate --output HDMI-A-1 --rate 1.5
```

### Configuration
```bash
wayvid-ctl reload   # Reload config from file
wayvid-ctl quit     # Stop daemon
```

### System Check
```bash
wayvid-ctl check    # Check system capabilities
```

## IPC Protocol

Unix socket: `$XDG_RUNTIME_DIR/wayvid.sock`

### Message Format

JSON-based request-response protocol:

**Request:**
```json
{"command": "get-status"}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "version": "0.4.4-alpha.2",
    "outputs": [...]
  }
}
```

### Available Commands

| Command | Parameters | Description |
|---------|------------|-------------|
| `get-status` | - | Get daemon status with all outputs |
| `pause` | `output?` | Pause playback |
| `resume` | `output?` | Resume playback |
| `seek` | `output`, `time` | Seek to time (seconds) |
| `set-volume` | `output`, `volume` | Set volume (0.0-1.0) |
| `toggle-mute` | `output` | Toggle mute |
| `set-layout` | `output`, `layout` | Set layout mode |
| `set-playback-rate` | `output`, `rate` | Set playback speed |
| `reload-config` | - | Reload config file |
| `quit` | - | Stop daemon |

## Programming Examples

### Python
```python
import socket
import json

def wayvid_command(command):
    sock = socket.socket(socket.AF_UNIX)
    sock.connect('/run/user/1000/wayvid.sock')
    sock.send((json.dumps(command) + '\n').encode())
    response = json.loads(sock.recv(4096).decode())
    sock.close()
    return response

# Get status
status = wayvid_command({"command": "get-status"})
for output in status.get("data", {}).get("outputs", []):
    print(f"{output['name']}: {output['width']}x{output['height']}")

# Pause all
wayvid_command({"command": "pause"})
```

### Bash
```bash
# Get status
echo '{"command":"get-status"}' | nc -U $XDG_RUNTIME_DIR/wayvid.sock | jq

# Pause
echo '{"command":"pause"}' | nc -U $XDG_RUNTIME_DIR/wayvid.sock
```

### Rust
```rust
use wayvid::ctl::ipc_client::IpcClient;
use wayvid::ctl::protocol::{IpcCommand, IpcResponse};

let mut client = IpcClient::connect()?;
let response = client.send_command(&IpcCommand::GetStatus)?;

if let IpcResponse::Success { data: Some(status) } = response {
    println!("{}", serde_json::to_string_pretty(&status)?);
}
```

## Troubleshooting

**Socket not found:**
```bash
# Check if daemon is running
ls $XDG_RUNTIME_DIR/wayvid.sock
systemctl --user status wayvid.service
```

**Permission denied:**
- Ensure wayvid-ctl runs as the same user as wayvid daemon

**No response / Timeout:**
```bash
# Check daemon logs
journalctl --user -u wayvid -f

# Restart daemon
systemctl --user restart wayvid.service
```

**GUI not showing outputs:**
- Ensure daemon is running first
- GUI auto-connects when daemon is detected
- Check status bar at bottom of GUI window

## See Also

- [IPC Protocol Reference](../reference/ipc-protocol.md) - Full protocol spec
- [CLI Commands](../reference/cli.md) - Command reference
- [Configuration](../user-guide/configuration.md) - Config options
