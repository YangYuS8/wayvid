# gui-integration Specification

## Purpose
TBD - created by archiving change implement-gui-daemon-ipc. Update Purpose after archive.
## Requirements
### Requirement: GUI-Daemon IPC Communication
The GUI application SHALL communicate with the wayvid daemon via Unix socket IPC using JSON-serialized messages defined in `wayvid_core::ipc`.

#### Scenario: Apply wallpaper from GUI
- **WHEN** user double-clicks a wallpaper in the library view
- **THEN** the GUI sends `IpcRequest::Apply` to the daemon
- **AND** the wallpaper is displayed on the target output

#### Scenario: Query daemon status
- **WHEN** the GUI starts or periodically polls
- **THEN** the GUI sends `IpcRequest::Status` to the daemon
- **AND** receives `IpcResponse::Status` with running state and output information

#### Scenario: Daemon not running
- **WHEN** the GUI attempts to connect and the daemon socket does not exist
- **THEN** the GUI displays "Daemon stopped" status in the sidebar
- **AND** enables the "Start Daemon" button

#### Scenario: Connection lost
- **WHEN** the daemon stops while GUI is connected
- **THEN** the GUI detects connection loss within 5 seconds
- **AND** attempts automatic reconnection with exponential backoff

### Requirement: Daemon Process Management
The GUI application SHALL be able to start and stop the wayvid daemon process.

#### Scenario: Start daemon from GUI
- **WHEN** user clicks "Start Daemon" button
- **THEN** the GUI spawns `wayvid` process in background
- **AND** waits for socket to become available
- **AND** updates status to "Daemon running"

#### Scenario: Stop daemon from GUI
- **WHEN** user clicks "Stop Daemon" button
- **THEN** the GUI sends `IpcRequest::Quit` to the daemon
- **AND** waits for daemon process to exit
- **AND** updates status to "Daemon stopped"

### Requirement: Real-time Monitor Information
The GUI application SHALL display real-time information about connected monitors from the daemon.

#### Scenario: List monitors from daemon
- **WHEN** user navigates to Monitors view
- **THEN** the GUI sends `IpcRequest::Outputs` to the daemon
- **AND** displays actual monitor names, resolutions, and positions

#### Scenario: Apply wallpaper to specific monitor
- **WHEN** user selects a monitor and clicks "Apply"
- **THEN** the GUI sends `IpcRequest::Apply` with specific output name
- **AND** the wallpaper is applied only to that monitor

### Requirement: Asynchronous Thumbnail Loading
The GUI application SHALL load wallpaper thumbnails asynchronously without blocking the main UI thread.

#### Scenario: Load thumbnail for visible wallpaper
- **WHEN** a wallpaper card becomes visible in the library view
- **THEN** the GUI requests thumbnail generation in a background task
- **AND** displays a loading placeholder until complete
- **AND** replaces placeholder with actual thumbnail when ready

#### Scenario: Workshop wallpaper preview
- **WHEN** a Steam Workshop wallpaper has a preview image (preview.jpg)
- **THEN** the GUI loads the preview image directly
- **AND** does not generate a video frame thumbnail

#### Scenario: Video thumbnail generation
- **WHEN** a local video wallpaper needs a thumbnail
- **AND** no cached thumbnail exists
- **THEN** the GUI extracts a frame from the video using ThumbnailGenerator
- **AND** caches the result to disk for future use

#### Scenario: Thumbnail loading failure
- **WHEN** thumbnail generation fails (corrupted file, unsupported format)
- **THEN** the GUI displays a fallback icon based on wallpaper type
- **AND** does not retry failed thumbnails repeatedly

### Requirement: Thumbnail Caching
The GUI application SHALL cache generated thumbnails to improve startup performance.

#### Scenario: Disk cache lookup
- **WHEN** a thumbnail is requested
- **THEN** the GUI first checks `~/.cache/wayvid/thumbnails/` for cached version
- **AND** uses cached version if file exists and is valid

#### Scenario: Cache persistence
- **WHEN** a thumbnail is generated
- **THEN** the GUI saves it to disk cache as WebP format
- **AND** the cached thumbnail survives application restarts

#### Scenario: Memory cache limit
- **WHEN** the in-memory thumbnail cache exceeds 100 entries
- **THEN** the GUI evicts least-recently-used thumbnails from memory
- **AND** keeps disk cache intact for future loads

### Requirement: Visibility-Based Loading
The GUI application SHALL prioritize thumbnail loading for visible wallpapers.

#### Scenario: Scroll-based loading
- **WHEN** user scrolls through the library
- **THEN** the GUI requests thumbnails for newly visible wallpapers
- **AND** cancels pending requests for wallpapers scrolled out of view

#### Scenario: Large library performance
- **WHEN** library contains 1000+ wallpapers
- **THEN** the GUI maintains smooth scrolling (60fps)
- **AND** only loads thumbnails for visible items plus buffer

### Requirement: Settings Persistence
The GUI application SHALL persist user settings to disk and restore them on startup.

#### Scenario: Save settings on change
- **WHEN** user changes any setting in the Settings view
- **THEN** the GUI persists the change to `~/.config/wayvid/settings.yaml`
- **AND** uses debounced saving (500ms) to avoid excessive disk writes

#### Scenario: Load settings on startup
- **WHEN** the GUI application starts
- **THEN** it loads settings from `~/.config/wayvid/settings.yaml`
- **AND** applies all settings to the initial application state

#### Scenario: Missing settings file
- **WHEN** the settings file does not exist
- **THEN** the GUI uses default settings
- **AND** creates the settings file on first setting change

#### Scenario: Corrupted settings file
- **WHEN** the settings file contains invalid YAML
- **THEN** the GUI logs a warning
- **AND** uses default settings
- **AND** does not crash

### Requirement: Autostart Management
The GUI application SHALL manage XDG autostart entries for automatic startup.

#### Scenario: Enable autostart
- **WHEN** user enables "Start with system" in settings
- **THEN** the GUI creates `~/.config/autostart/wayvid.desktop`
- **AND** the desktop entry launches wayvid-gui with --minimized flag

#### Scenario: Disable autostart
- **WHEN** user disables "Start with system" in settings
- **THEN** the GUI removes `~/.config/autostart/wayvid.desktop`

#### Scenario: Autostart state reflection
- **WHEN** user opens Settings view
- **THEN** the autostart toggle reflects actual file existence
- **AND** handles external modifications to the desktop entry

### Requirement: Language Setting Persistence
The GUI application SHALL persist the user's language preference.

#### Scenario: Language change persistence
- **WHEN** user changes language in settings
- **THEN** the GUI saves the language preference
- **AND** applies the new language immediately

#### Scenario: Language restoration on startup
- **WHEN** the GUI starts with a saved language preference
- **THEN** it applies the saved language before rendering UI
- **AND** "System" option uses system locale detection

### Requirement: Power Management Settings
The GUI application SHALL respect power management settings for playback control.

#### Scenario: Pause on battery
- **WHEN** "Pause on battery" is enabled
- **AND** system switches to battery power
- **THEN** the GUI sends pause command to daemon

#### Scenario: Resume on AC power
- **WHEN** "Pause on battery" is enabled
- **AND** system switches to AC power
- **THEN** the GUI sends resume command to daemon

#### Scenario: Battery state detection
- **WHEN** the GUI is running
- **THEN** it monitors `/sys/class/power_supply/*/status` for battery state changes
- **AND** detects changes within 30 seconds

### Requirement: Window State Persistence
The GUI application SHALL remember window dimensions across sessions.

#### Scenario: Save window size
- **WHEN** user resizes the application window
- **THEN** the GUI saves the new dimensions to settings

#### Scenario: Restore window size
- **WHEN** the GUI starts
- **THEN** it restores the previously saved window dimensions
- **AND** validates dimensions are within screen bounds

