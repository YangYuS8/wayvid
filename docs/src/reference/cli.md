# CLI Commands

Complete reference for all CLI commands.

## wayvid

Main daemon and management tool.

```
wayvid <COMMAND> [OPTIONS]
```

### Commands

#### run
Run the wallpaper engine daemon.
```bash
wayvid run [--config <PATH>]

# Examples
wayvid run
wayvid run --config ~/.config/wayvid/custom.yaml
wayvid run --log-level debug
```

#### daemon
Manage wayvid daemon (via systemd).
```bash
# Start daemon
wayvid daemon start

# Stop daemon
wayvid daemon stop

# Restart daemon
wayvid daemon restart

# Check status
wayvid daemon status

# View logs
wayvid daemon logs
wayvid daemon logs --follow
wayvid daemon logs --lines 100
```

**Note:** These commands use systemd user services. Ensure `wayvid.service` is installed.

#### check
Check system capabilities and dependencies.
```bash
wayvid check
```

#### import
Import Wallpaper Engine project.
```bash
wayvid import <PROJECT_DIR> [--output <PATH>]

# Example
wayvid import ~/steamapps/workshop/content/431960/1234567890 -o config.yaml
```

### Global Options
- `--log-level <LEVEL>` - Log level (error, warn, info, debug, trace)
- `--version` - Print version
- `--help` - Show help

### Examples
```bash
# Start daemon with systemd
wayvid daemon start

# Run directly with custom config
wayvid run --config ~/my-config.yaml

# Debug logging
wayvid run --log-level debug

# Check system capabilities
wayvid check
```

## wayvid-ctl

IPC control tool.

```
wayvid-ctl <COMMAND> [OPTIONS]
```

### Commands

#### status
Show current status.
```bash
wayvid-ctl status
```

#### play
Start playback.
```bash
wayvid-ctl play [--output <NAME>]
```

#### pause
Pause playback.
```bash
wayvid-ctl pause [--output <NAME>]
```

#### stop
Stop playback.
```bash
wayvid-ctl stop [--output <NAME>]
```

#### resume
Resume playback.
```bash
wayvid-ctl resume [--output <NAME>]
```

#### reload-config
Reload configuration.
```bash
wayvid-ctl reload-config
```

#### set-volume
Set volume (0-100).
```bash
wayvid-ctl set-volume <VOLUME>
```

#### set-source
Change video source.
```bash
wayvid-ctl set-source <PATH>
```

#### list-outputs
List all outputs.
```bash
wayvid-ctl list-outputs
```

### Global Options
- `--socket <PATH>` - IPC socket path
- `--help` - Show help

## wayvid workshop

Workshop management (requires `--features workshop`).

```
wayvid workshop <COMMAND>
```

### Commands

#### list
List all workshop items.
```bash
wayvid workshop list
```

#### info
Show item details.
```bash
wayvid workshop info <ID>
```

#### import
Import item to config.
```bash
wayvid workshop import <ID>
```

## wayvid-gui

Desktop GUI (requires `--features gui`).

```
wayvid-gui
```

No command-line options. Launch GUI directly.

## Environment Variables

- `RUST_LOG` - Log level (overrides `--log-level`)
- `XDG_RUNTIME_DIR` - IPC socket location
- `WAYLAND_DISPLAY` - Wayland display name
