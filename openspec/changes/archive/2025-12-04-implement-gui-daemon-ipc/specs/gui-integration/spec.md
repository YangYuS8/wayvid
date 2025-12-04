# GUI Integration - IPC Communication

## ADDED Requirements

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
