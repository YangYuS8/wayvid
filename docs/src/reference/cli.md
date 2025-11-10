# CLI Commands

Complete reference for all CLI commands.

## wayvid

Main daemon.

```
wayvid [OPTIONS]
```

### Options
- `--config <PATH>` - Config file path
- `--generate-config` - Generate default config
- `--log-level <LEVEL>` - Log level (error, warn, info, debug, trace)
- `--version` - Print version
- `--help` - Show help

### Examples
```bash
# Run with custom config
wayvid --config ~/my-config.yaml

# Generate config
wayvid --generate-config

# Debug logging
wayvid --log-level debug
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
