# GUI Control Panel

wayvid includes a graphical control panel for easy management.

## Starting the GUI

```bash
wayvid-gui
```

**Note:** The GUI is now **fully decoupled** from the daemon. It functions as a:
- 📝 **Configuration Editor** - Visual interface for config.yaml
- 🎛️ **Control Panel** - Runtime control via IPC
- 📊 **Status Monitor** - Real-time daemon status
- 🔧 **Daemon Manager** - Start/stop/restart daemon

Or build with GUI support:
```bash
cargo build --release --bin wayvid-gui --features gui
```

## Daemon Management

The GUI can now **manage the daemon lifecycle**:

### Top Panel Controls

**When daemon is running:**
- ⏹ **Stop Daemon** - Stop wayvid service
- 🔄 **Restart** - Restart daemon (useful after config changes)

**When daemon is NOT running:**
- 🚀 **Start Daemon** - Start wayvid via systemd

**Connection Status:**
- 🟢 **● Connected** - GUI is connected to daemon
- 🟡 **● Connecting...** - Attempting connection
- ⚪ **● Disconnected** - Not connected
- 🔴 **● Error** - Connection error

**If daemon is not running**, the GUI displays:
```
⚠ wayvid daemon is not running. Start it using the button above or run:
   systemctl --user start wayvid.service
```

### Typical Usage

**Option 1: GUI manages daemon (Easiest)**
1. Launch `wayvid-gui`
2. Click **🚀 Start Daemon**
3. GUI auto-connects when daemon is ready

**Option 2: Manual daemon, GUI for control**
1. Start daemon: `systemctl --user start wayvid.service`
2. Launch `wayvid-gui`
3. GUI auto-connects if daemon is running

**Option 3: Fully independent**
1. Daemon runs as systemd service (always-on)
2. Open GUI when you need to change settings
3. Close GUI when done (daemon keeps running)

## Features

### 📺 Outputs Tab

Manage connected displays:
- View all detected outputs (monitors)
- Monitor resolution and status
- Pause/Resume playback per output
- Select outputs for configuration

**Actions:**
- Click checkbox to select an output
- Use **Pause/Resume** buttons for playback control
- Click **Configure** for per-output settings

### 🎬 Video Sources Tab

Add and manage video sources:

**Local Files:**
- Enter file path or drag & drop
- Browse common directories (Videos, Pictures, Downloads)
- Apply to selected output

**Stream URLs:**
- HTTP(S), RTSP, HLS, DASH support
- Enter URL and apply to output

**Quick Access:**
- One-click access to ~/Videos, ~/Pictures, ~/Downloads
- Recent sources history

### 🎮 Workshop Tab

Steam Workshop integration:

**Features:**
- **Scan Workshop**: Detect installed Wallpaper Engine items
- **Search**: Filter by title or ID
- **Grid View**: Browse wallpapers visually
- **Preview**: Open video in Sources tab
- **Import**: One-click config generation

**Status Indicators:**
- ✓ Valid video wallpaper (green)
- ⚠ No video or invalid (yellow)

**Actions:**
- **▶ Preview**: Load video into Sources tab
- **📥 Import**: Generate config for the Workshop item

### ⚙ Settings Tab

Configure playback and application:

**Video Configuration:**
- **Layout Mode**: Fill, Contain, Stretch, Cover, Centre
  - *Fill*: Scale to cover screen, crop edges (recommended)
  - *Contain*: Fit inside screen (may have black bars)
  - *Stretch*: Fill screen (may distort)
  - *Centre*: Original size, centered
- **Loop playback**: Enable/disable looping
- **Hardware decoding**: VA-API/NVDEC support
- **Mute**: Toggle audio
- **Volume**: 0-100% slider

**Actions:**
- **Apply to Selected Output**: Update output configuration
- **Save as Config File**: Export to YAML (future)

**Performance:**
- Max FPS: Unlimited (vsync)
- Memory limit: 100 MB (default)
- Decode mode: Shared (optimal)

**About:**
- Version information
- Links to GitHub and documentation

## Typical Workflow

### Quick Start (GUI Manages Everything)

1. **Launch GUI:**
   ```bash
   wayvid-gui
   ```

2. **Start daemon (if not running):**
   - If you see "⚠ wayvid daemon is not running"
   - Click **🚀 Start Daemon** button
   - Wait for status to show **● Connected**

3. **Or start daemon manually first:**
   ```bash
   systemctl --user start wayvid.service
   # Then launch GUI
   wayvid-gui
   ```

### Configure Wallpaper

4. **Select an output:**
   - Go to **📺 Outputs** tab
   - Click checkbox next to desired monitor

5. **Choose video source:**
   
   **Option A - Local File:**
   - Go to **🎬 Video Sources** tab
   - Enter path or use Quick Access
   - Click **✓ Apply to Selected Output**
   
   **Option B - Workshop:**
   - Go to **🎮 Workshop** tab
   - Click **🔄 Scan Workshop**
   - Find desired wallpaper
   - Click **📥 Import**
   - Go back to Sources tab and apply

6. **Configure playback:**
   - Go to **⚙ Settings** tab
   - Choose Layout Mode (Fill recommended)
   - Adjust volume, loop, etc.
   - Click **💾 Apply to Selected Output**

## Requirements

- wayvid daemon must be running
- IPC socket at `/run/user/$UID/wayvid.sock`
- For Workshop: Steam + Wallpaper Engine installed

## Keyboard Shortcuts

- `Ctrl+Q`: Quit GUI
- `F5`: Refresh outputs
- `Esc`: Deselect output

## Troubleshooting

### "wayvid daemon not running"
**Solution 1: Use GUI button**
- Click **🚀 Start Daemon** in top panel

**Solution 2: Start via systemd**
```bash
systemctl --user start wayvid.service
```

**Solution 3: Start manually**
```bash
wayvid run &
```

**Check if daemon is running:**
```bash
# Via systemctl
systemctl --user status wayvid.service

# Or check process
ps aux | grep wayvid
```

### "No outputs detected"
1. Check if wayvid is running: `ps aux | grep wayvid`
2. Click **🔄 Refresh** button
3. Check logs: `journalctl --user -u wayvid -f`

### "No Workshop items found"
Ensure:
- Steam is installed
- Wallpaper Engine is in your library
- You have subscribed to Workshop items
- Items are downloaded (check Steam Workshop tab)

### GUI doesn't start
Build with GUI support:
```bash
cargo build --release --bin wayvid-gui --features gui
```

## Architecture

The GUI uses:
- **egui/eframe**: Immediate mode GUI framework
- **IPC**: Unix socket communication with daemon
- **Threading**: Async IPC communication

Communication flow:
```
GUI → IPC Socket → wayvid daemon → Wayland compositor
```

## Future Enhancements

Planned features:
- [ ] Native file dialog (requires rfd crate)
- [ ] Drag & drop file support
- [ ] System tray icon
- [ ] Auto-connect on startup
- [ ] Live video preview
- [ ] Config file editor with syntax highlighting
- [ ] Workshop thumbnail display
- [ ] Playlist management
- [ ] Per-output timeline/scrubbing

## See Also

- [Configuration Guide](configuration.md)
- [CLI Control](../reference/cli.md)
- [IPC Protocol](../reference/ipc.md)
