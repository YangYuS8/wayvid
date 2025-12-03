# IPC Protocol

JSON over Unix socket at `$XDG_RUNTIME_DIR/wayvid.sock`.

## Commands

```json
{"command": "get-status"}
{"command": "pause"}
{"command": "resume"}
{"command": "pause", "output": "DP-1"}
{"command": "set-source", "source": "/path/to/video.mp4"}
{"command": "set-volume", "volume": 0.5}
{"command": "reload-config"}
{"command": "quit"}
```

## Response

```json
{"status": "success", "data": {...}}
{"status": "error", "message": "Error description"}
```

## Example (Python)

```python
import socket, json

sock = socket.socket(socket.AF_UNIX)
sock.connect('/run/user/1000/wayvid.sock')
sock.send(b'{"command":"get-status"}\n')
print(sock.recv(4096).decode())
sock.close()
```

## Example (Bash)

```bash
echo '{"command":"get-status"}' | nc -U $XDG_RUNTIME_DIR/wayvid.sock
```
