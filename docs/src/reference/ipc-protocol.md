# IPC Protocol

wayvid's IPC protocol specification.

## Transport

**Protocol**: JSON over Unix socket  
**Socket**: `$XDG_RUNTIME_DIR/wayvid.sock`  
**Format**: Newline-delimited JSON (request-response)

## Message Format

### Request
```json
{
  "command": "get-status"
}
```

Commands use **kebab-case** naming convention.

### Response
```json
{
  "status": "success",
  "data": {}
}
```

Or for errors:
```json
{
  "status": "error",
  "message": "Error description"
}
```

## Commands

### get-status
Get daemon status including all outputs.

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
    "outputs": [
      {
        "name": "HDMI-A-1",
        "width": 2560,
        "height": 1440,
        "playing": true,
        "paused": false,
        "current_time": 12.5,
        "duration": 120.0,
        "source": "/path/to/video.mp4",
        "layout": "Fill",
        "volume": 0.5,
        "muted": true,
        "playback_rate": 1.0
      }
    ]
  }
}
```

### pause
Pause playback on specific or all outputs.

**Request:**
```json
{
  "command": "pause",
  "output": "HDMI-A-1"
}
```

Or pause all:
```json
{"command": "pause"}
```

### resume
Resume playback.

**Request:**
```json
{
  "command": "resume",
  "output": "HDMI-A-1"
}
```

### seek
Seek to specific time (seconds).

**Request:**
```json
{
  "command": "seek",
  "output": "HDMI-A-1",
  "time": 30.5
}
```

### switch-source
Change video source for an output.

**Request:**
```json
{
  "command": "switch-source",
  "output": "HDMI-A-1",
  "source": {
    "type": "File",
    "path": "/path/to/new-video.mp4"
  }
}
```

### set-source
Set video source (simplified, path only).

**Request:**
```json
{
  "command": "set-source",
  "output": "HDMI-A-1",
  "source": "/path/to/video.mp4"
}
```

### reload-config
Reload configuration from file.

**Request:**
```json
{"command": "reload-config"}
```

### set-playback-rate
Set playback speed.

**Request:**
```json
{
  "command": "set-playback-rate",
  "output": "HDMI-A-1",
  "rate": 1.5
}
```

### set-volume
Set volume (0.0 - 1.0).

**Request:**
```json
{
  "command": "set-volume",
  "output": "HDMI-A-1",
  "volume": 0.8
}
```

### toggle-mute
Toggle audio mute.

**Request:**
```json
{
  "command": "toggle-mute",
  "output": "HDMI-A-1"
}
```

### set-layout
Set layout mode.

**Request:**
```json
{
  "command": "set-layout",
  "output": "HDMI-A-1",
  "layout": "fill"
}
```

Available layouts: `fill`, `contain`, `stretch`, `cover`, `centre`

### quit
Stop the daemon.

**Request:**
```json
{"command": "quit"}
```

## Error Handling

### Error Response
```json
{
  "status": "error",
  "message": "Output 'DP-1' not found"
}
```

### Common Errors
- `"Timeout waiting for daemon response"` - Daemon busy or unresponsive
- `"Failed to connect to daemon"` - Daemon not running
- `"Failed to parse command"` - Invalid JSON syntax
- `"Output not found"` - Specified output doesn't exist

## Example Implementations

### Python
```python
import socket
import json

def send_command(command):
    sock = socket.socket(socket.AF_UNIX)
    sock.connect('/run/user/1000/wayvid.sock')
    
    sock.send((json.dumps(command) + '\n').encode())
    response = sock.recv(4096).decode()
    sock.close()
    
    return json.loads(response)

# Get status
status = send_command({"command": "get-status"})
print(status)

# Pause playback
send_command({"command": "pause"})
```

### Bash
```bash
echo '{"command":"get-status"}' | nc -U $XDG_RUNTIME_DIR/wayvid.sock
```

### Rust
```rust
use wayvid::ctl::protocol::{IpcCommand, IpcResponse};
use wayvid::ctl::ipc_client::IpcClient;

let mut client = IpcClient::connect()?;
let response = client.send_command(&IpcCommand::GetStatus)?;

match response {
    IpcResponse::Success { data } => {
        if let Some(status) = data {
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
    }
    IpcResponse::Error { message } => {
        eprintln!("Error: {}", message);
    }
}
```

## Architecture

The IPC system uses a request-response pattern:

1. Client connects to Unix socket
2. Client sends JSON command (newline-terminated)
3. Daemon processes command and generates response
4. Daemon sends JSON response (newline-terminated)
5. Connection can be reused for multiple commands

The daemon uses non-blocking event polling to handle IPC requests alongside Wayland events, ensuring responsive command processing.

## See Also

- [IPC Control](../features/ipc.md) - User guide
- [CLI Commands](./cli.md) - Command-line interface
