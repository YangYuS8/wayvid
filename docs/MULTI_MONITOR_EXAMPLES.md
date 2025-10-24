# Multi-Monitor Configuration Examples

This document provides examples of using wayvid's advanced multi-monitor features with pattern matching and priority-based selection.

## Table of Contents

- [Configuration Patterns](#configuration-patterns)
- [Priority System](#priority-system)
- [Runtime Control](#runtime-control)
- [Common Scenarios](#common-scenarios)

## Configuration Patterns

### Basic Pattern Matching

Use glob-style wildcards in your configuration:

```yaml
per_output:
  # Exact match - highest priority (implicit priority: 0)
  "eDP-1":
    source:
      type: File
      path: /home/user/videos/laptop-bg.mp4
    layout: fill

  # Pattern with * wildcard
  "HDMI-*":
    source:
      type: File
      path: /home/user/videos/external-bg.mp4
    layout: cover

  # Pattern with ? wildcard (single character)
  "DP-?":
    source:
      type: File
      path: /home/user/videos/displayport.mp4
    
  # Fallback for any output
  "*":
    source:
      type: File
      path: /home/user/videos/default.mp4
```

### Pattern Wildcards

- `*` - Matches zero or more characters
- `?` - Matches exactly one character
- Exact names always take precedence

**Examples:**
- `HDMI-*` matches `HDMI-A-1`, `HDMI-A-2`, `HDMI-B-1`
- `DP-?` matches `DP-1`, `DP-2` but not `DP-10`
- `*` matches anything

## Priority System

When multiple patterns match an output, priority determines which configuration is used.

### Priority Rules

1. **Exact matches always win** (implicit priority: 0)
2. **Lower priority values = higher precedence**
3. **Default priority: 50** (if not specified)
4. Within same priority, more specific patterns win

### Priority Configuration

```yaml
per_output:
  # Specific HDMI-A monitors (high priority)
  "HDMI-A-*":
    priority: 5
    source:
      type: File
      path: /home/user/videos/hdmi-a-specific.mp4

  # All other HDMI monitors (lower priority)
  "HDMI-*":
    priority: 10
    source:
      type: File
      path: /home/user/videos/hdmi-general.mp4

  # All DisplayPort monitors
  "DP-*":
    priority: 10
    source:
      type: Url
      url: https://example.com/stream.m3u8

  # Fallback for everything else (lowest priority)
  "*":
    priority: 99
    source:
      type: File
      path: /home/user/videos/fallback.mp4
```

### Selection Algorithm

For output `HDMI-A-1`:
1. ✅ `HDMI-A-1` exact match → priority 0 (wins if exists)
2. ✅ `HDMI-A-*` pattern → priority 5
3. ✅ `HDMI-*` pattern → priority 10
4. ✅ `*` pattern → priority 99

**Result:** Uses `HDMI-A-*` configuration (priority 5)

## Runtime Control

### Dynamic Source Switching

Use `wayvid-ctl` to switch video sources at runtime:

```bash
# Switch to a local file
wayvid-ctl switch -o eDP-1 file:///home/user/videos/new-video.mp4

# Or use absolute path (file:// is implicit)
wayvid-ctl switch -o eDP-1 /home/user/videos/new-video.mp4

# Switch to HTTP stream
wayvid-ctl switch -o HDMI-A-1 https://example.com/stream.m3u8

# Switch to RTSP stream
wayvid-ctl switch -o DP-1 rtsp://camera.local/stream

# Switch to pipe input (stdin)
wayvid-ctl switch -o eDP-1 pipe://
cat video.mp4 | wayvid
```

### Other Commands

```bash
# Get status of all outputs
wayvid-ctl status

# Pause/resume playback
wayvid-ctl pause -o eDP-1
wayvid-ctl resume -o eDP-1

# Set playback rate (speed)
wayvid-ctl rate -o eDP-1 1.5

# Set volume (0.0 - 1.0)
wayvid-ctl volume -o eDP-1 0.8

# Toggle mute
wayvid-ctl mute -o eDP-1

# Set layout mode
wayvid-ctl layout -o eDP-1 cover

# Reload configuration
wayvid-ctl reload

# Quit daemon
wayvid-ctl quit
```

## Common Scenarios

### Scenario 1: Laptop + External Monitor

Different videos for laptop and external displays:

```yaml
per_output:
  "eDP-1":  # Laptop screen
    source:
      type: File
      path: /home/user/videos/personal.mp4
    layout: fill
    
  "HDMI-*":  # Any external HDMI
    priority: 5
    source:
      type: File
      path: /home/user/videos/professional.mp4
    layout: cover
    
  "DP-*":  # Any external DisplayPort
    priority: 5
    source:
      type: Url
      url: https://stream.example.com/feed.m3u8
    layout: cover
```

### Scenario 2: Multi-Monitor Workstation

Different content per monitor type with fallback:

```yaml
per_output:
  # Left monitor (specific)
  "DP-1":
    source:
      type: File
      path: /home/user/videos/left-ambient.mp4
    layout: cover
    
  # Center monitor (specific)
  "DP-2":
    source:
      type: File
      path: /home/user/videos/center-focus.mp4
    layout: fill
    
  # Right monitor (specific)
  "DP-3":
    source:
      type: File
      path: /home/user/videos/right-ambient.mp4
    layout: cover
    
  # Fallback for any hot-plugged monitor
  "*":
    priority: 99
    source:
      type: File
      path: /home/user/videos/default-elegant.mp4
    layout: contain
```

### Scenario 3: Priority-Based Fallback

Layered priority system for different monitor brands:

```yaml
per_output:
  # High-end monitors get 4K content
  "Dell-*":
    priority: 5
    source:
      type: File
      path: /home/user/videos/4k-content.mp4
    layout: fill
    
  # Standard monitors get 1080p
  "HDMI-*":
    priority: 10
    source:
      type: File
      path: /home/user/videos/1080p-content.mp4
    layout: cover
    
  # Generic DisplayPort
  "DP-*":
    priority: 10
    source:
      type: File
      path: /home/user/videos/1080p-content.mp4
    layout: cover
    
  # Ultimate fallback
  "*":
    priority: 99
    source:
      type: File
      path: /home/user/videos/safe-default.mp4
    layout: contain
```

### Scenario 4: Development vs Production

Use different configs for different environments:

**Development (`~/.config/wayvid/config.yaml`):**
```yaml
per_output:
  "*":
    source:
      type: File
      path: /home/user/videos/test-pattern.mp4
    layout: contain
    playback_rate: 2.0  # Fast playback for testing
```

**Production (switch at runtime):**
```bash
# Switch all monitors to production content
wayvid-ctl switch -o eDP-1 /srv/videos/production-bg.mp4
wayvid-ctl switch -o HDMI-A-1 /srv/videos/production-bg.mp4
wayvid-ctl rate -o eDP-1 1.0  # Normal speed
```

### Scenario 5: Hot-Plug Handling

Configuration that gracefully handles monitor hot-plugging:

```yaml
per_output:
  # Permanent monitors (exact names)
  "eDP-1":
    source:
      type: File
      path: /home/user/videos/laptop.mp4
      
  "DP-1":
    source:
      type: File
      path: /home/user/videos/primary-external.mp4
  
  # Hot-pluggable monitors by type
  "HDMI-A-*":
    priority: 10
    source:
      type: File
      path: /home/user/videos/hdmi-hotplug.mp4
      
  # Catch-all for unknown monitors
  "*":
    priority: 99
    source:
      type: File
      path: /home/user/videos/fallback.mp4
    layout: contain  # Safe layout for unknown aspect ratios
```

## Best Practices

1. **Always provide a fallback:**
   ```yaml
   "*":
     priority: 99
     source: { type: File, path: /path/to/safe-default.mp4 }
   ```

2. **Use priorities to layer specificity:**
   - Exact names: priority 0 (implicit)
   - Brand/model patterns: priority 5-10
   - Generic patterns: priority 10-50
   - Fallback: priority 99

3. **Test pattern matching:**
   ```bash
   # Check which config applies to an output
   wayvid-ctl status | jq '.outputs[] | select(.name=="HDMI-A-1")'
   ```

4. **Monitor hot-plug events:**
   - Always include generic patterns (`HDMI-*`, `DP-*`, `*`)
   - Use `contain` layout for fallbacks (safe for any aspect ratio)

5. **Use runtime switching for temporary changes:**
   ```bash
   # Don't edit config for one-time changes
   wayvid-ctl switch -o eDP-1 /tmp/temporary-video.mp4
   # Reload config to restore configured source
   wayvid-ctl reload
   ```

## Troubleshooting

### Pattern not matching?

Check output names:
```bash
wayvid-ctl status | jq -r '.outputs[].name'
```

### Priority conflicts?

Test manually:
```bash
# Override with exact name
wayvid-ctl switch -o "HDMI-A-1" /path/to/test.mp4
```

### Performance issues?

- Avoid too many patterns (keep under 20)
- Use exact matches for known monitors
- Profile with `wayvid --log-level debug`

## See Also

- [Configuration Schema](../README.md#configuration)
- [IPC Protocol](./M5_ISSUE2_PROGRESS.md)
- [Pattern Matching Implementation](../src/config/pattern.rs)
