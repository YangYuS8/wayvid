# gui-integration Specification (Delta)

## MODIFIED Requirements

### Requirement: GUI-Daemon IPC Communication
**Reason**: v0.5 uses integrated engine, not external daemon

~~The GUI application SHALL communicate with the wayvid daemon via Unix socket IPC using JSON-serialized messages defined in `wayvid_core::ipc`.~~

The GUI application SHALL embed the playback engine directly, using internal channels for communication instead of external IPC.

#### Scenario: Apply wallpaper from GUI
- **WHEN** user double-clicks a wallpaper in the library view
- **THEN** the GUI sends `EngineCommand::ApplyWallpaper` to the embedded engine
- **AND** the wallpaper is displayed on the target output

#### Scenario: Query engine status
- **WHEN** the GUI needs current playback status
- **THEN** the GUI receives `EngineEvent` messages from the engine subscription
- **AND** updates UI to reflect current state

#### Scenario: Engine not started
- **WHEN** the embedded engine is not running
- **THEN** the GUI displays "Engine stopped" status in the sidebar
- **AND** enables the "Start Engine" button

#### Scenario: Engine error
- **WHEN** the engine encounters a fatal error
- **THEN** the engine sends `EngineEvent::Error`
- **AND** the GUI displays the error message
- **AND** allows user to restart the engine

### Requirement: Daemon Process Management
**Renamed to**: Engine Lifecycle Management

~~The GUI application SHALL be able to start and stop the wayvid daemon process.~~

The GUI application SHALL manage the embedded playback engine lifecycle.

#### Scenario: Start engine from GUI
- **WHEN** user clicks "Start Engine" button
- **THEN** the GUI initializes `PlaybackEngine` in a dedicated thread
- **AND** establishes communication channels
- **AND** updates status to "Engine running"

#### Scenario: Stop engine from GUI
- **WHEN** user clicks "Stop Engine" button
- **THEN** the GUI sends shutdown command to engine
- **AND** engine cleans up all surfaces and resources
- **AND** engine thread exits gracefully
- **AND** GUI updates status to "Engine stopped"

#### Scenario: Auto-start engine
- **WHEN** GUI starts with auto-start enabled
- **THEN** the engine SHALL be initialized automatically
- **AND** previous wallpaper configuration SHALL be restored

### Requirement: Real-time Monitor Information
**Modified**: Monitor info comes from engine's Wayland backend

~~The GUI application SHALL display real-time information about connected monitors from the daemon.~~

The GUI application SHALL display monitor information from the embedded engine's Wayland output detection.

#### Scenario: List monitors from engine
- **WHEN** user navigates to Monitors view
- **THEN** the GUI displays outputs detected by the engine
- **AND** falls back to wlr-randr when engine is not running

#### Scenario: Output hotplug notification
- **WHEN** a monitor is connected or disconnected
- **THEN** the engine sends `EngineEvent::OutputAdded` or `EngineEvent::OutputRemoved`
- **AND** the GUI updates the monitors list automatically

#### Scenario: Apply wallpaper to specific monitor
- **WHEN** user selects a monitor and clicks "Apply"
- **THEN** the GUI sends `EngineCommand::ApplyWallpaper` with specific output name
- **AND** the wallpaper is applied only to that monitor

## ADDED Requirements

### Requirement: Engine Thread Communication
The GUI SHALL communicate with the engine thread using asynchronous channels.

#### Scenario: Send command to engine
- **WHEN** the GUI needs to control playback
- **THEN** the GUI sends an `EngineCommand` via the command channel
- **AND** does not block the UI thread

#### Scenario: Receive event from engine
- **WHEN** the engine state changes
- **THEN** the engine sends an `EngineEvent` via the event channel
- **AND** the GUI receives it through an iced subscription

#### Scenario: Channel disconnection
- **WHEN** the engine thread terminates unexpectedly
- **THEN** the channel becomes disconnected
- **AND** the GUI detects this and updates status to "Engine stopped"

### Requirement: Built-in IPC Server (Optional)
The GUI SHALL optionally provide a Unix socket IPC server to allow external control via `wayvid-ctl`.

#### Scenario: wayvid-ctl sends command
- **WHEN** `wayvid-ctl apply` is executed
- **THEN** it connects to the GUI's IPC socket
- **AND** sends `IpcRequest::Apply`
- **AND** the GUI translates this to `EngineCommand::ApplyWallpaper`
- **AND** returns `IpcResponse::Ok` to wayvid-ctl

#### Scenario: IPC server disabled
- **WHEN** IPC server is disabled in settings
- **THEN** no Unix socket is created
- **AND** `wayvid-ctl` commands fail with "connection refused"

## REMOVED Requirements

### Requirement: External Daemon Process Spawning
**Reason**: v0.5 uses integrated engine

The GUI SHALL NOT spawn external `wayvid` daemon processes.

**Migration**: All daemon management code is replaced with internal engine lifecycle management.
