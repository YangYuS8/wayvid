# CLI Commands

## wayvid

Main daemon.

```bash
wayvid run                    # Run daemon
wayvid run --config <path>    # Custom config
wayvid daemon start           # Start via systemd
wayvid daemon stop            # Stop daemon
wayvid daemon status          # Check status
wayvid daemon logs --follow   # View logs
wayvid check                  # Check system
```

## wayvid-ctl

Runtime control (requires daemon running).

```bash
wayvid-ctl status             # Show status
wayvid-ctl play               # Play
wayvid-ctl pause              # Pause
wayvid-ctl resume             # Resume
wayvid-ctl set-source <path>  # Change video
wayvid-ctl set-volume <0-100> # Set volume
wayvid-ctl reload-config      # Reload config
wayvid-ctl list-outputs       # List monitors
```

Per-output control:
```bash
wayvid-ctl pause --output DP-1
wayvid-ctl set-source ~/vid.mp4 --output DP-1
```

## wayvid workshop

Steam Workshop management.

```bash
wayvid workshop list          # List subscribed
wayvid workshop info <id>     # Show details
wayvid workshop import <id>   # Import to config
wayvid workshop download <id> # Download item
```

## wayvid-gui

Desktop GUI (no arguments).

```bash
wayvid-gui
```
