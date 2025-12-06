# IPC Protocol

wayvid uses JSON over Unix socket for inter-process communication.

**Socket path:** `$XDG_RUNTIME_DIR/wayvid.sock` (typically `/run/user/1000/wayvid.sock`)

## Request Format

All requests are JSON objects with a `type` field indicating the command.

### Ping

Check if daemon is alive.

```json
{"type": "ping"}
```

Response:
```json
{"type": "pong"}
```

### Status

Get current daemon status.

```json
{"type": "status"}
```

Response:
```json
{
  "type": "status",
  "running": true,
  "version": "0.5.0-alpha.1",
  "outputs": [
    {
      "name": "eDP-1",
      "wallpaper": "/home/user/Videos/wallpaper.mp4",
      "paused": false,
      "volume": 1.0
    }
  ]
}
```

### Outputs

List available monitors.

```json
{"type": "outputs"}
```

Response:
```json
{
  "type": "outputs",
  "outputs": [
    {
      "name": "eDP-1",
      "width": 1920,
      "height": 1080,
      "refresh": 60,
      "primary": true,
      "x": 0,
      "y": 0
    }
  ]
}
```

### Apply

Apply wallpaper to output(s).

```json
{
  "type": "apply",
  "path": "/path/to/video.mp4",
  "output": "eDP-1",
  "mode": "fill"
}
```

- `output`: Optional. If omitted, applies to all outputs.
- `mode`: Optional. One of `fill`, `contain`, `stretch`, `centre`. Default: `fill`.

Response:
```json
{"type": "ok", "message": "Wallpaper applied"}
```

### Pause/Resume

```json
{"type": "pause", "output": "eDP-1"}
{"type": "resume", "output": "eDP-1"}
```

- `output`: Optional. If omitted, affects all outputs.

### Stop

Stop playback and clear wallpaper.

```json
{"type": "stop", "output": "eDP-1"}
```

### Set Volume

```json
{
  "type": "set_volume",
  "output": "eDP-1",
  "volume": 0.5
}
```

- `volume`: Float from 0.0 to 1.0.

### Quit

Request daemon shutdown.

```json
{"type": "quit"}
```

## Response Format

### Success

```json
{"type": "ok", "message": "Operation completed"}
```

### Error

```json
{"type": "error", "error": "Error description"}
```

## Examples

### Python

```python
import socket
import json

def send_ipc(request):
    sock = socket.socket(socket.AF_UNIX)
    sock.connect('/run/user/1000/wayvid.sock')
    sock.send((json.dumps(request) + '\n').encode())
    response = sock.recv(4096).decode()
    sock.close()
    return json.loads(response)

# Check status
status = send_ipc({"type": "status"})
print(f"Running: {status['running']}")
for output in status.get('outputs', []):
    print(f"  {output['name']}: {output.get('wallpaper', 'None')}")

# Apply wallpaper
result = send_ipc({
    "type": "apply",
    "path": "/home/user/Videos/wallpaper.mp4"
})
print(result)
```

### Bash

```bash
# Check status
echo '{"type":"status"}' | nc -U $XDG_RUNTIME_DIR/wayvid.sock

# Apply wallpaper
echo '{"type":"apply","path":"/home/user/Videos/wallpaper.mp4"}' | \
  nc -U $XDG_RUNTIME_DIR/wayvid.sock

# Pause all
echo '{"type":"pause"}' | nc -U $XDG_RUNTIME_DIR/wayvid.sock
```

### Using wayvid-ctl

For most use cases, the `wayvid-ctl` CLI tool is recommended:

```bash
wayvid-ctl status
wayvid-ctl apply ~/Videos/wallpaper.mp4
wayvid-ctl pause
```
