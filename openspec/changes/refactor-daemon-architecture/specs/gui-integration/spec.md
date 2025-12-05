# gui-integration Spec Delta: Daemon Architecture Refactoring

## MODIFIED Requirements

### Requirement: Daemon Process Management → Playback Engine Management
The GUI application SHALL manage the internal playback engine directly without spawning external processes.

**Reason**: v0.5 architecture integrates the daemon into wayvid-gui. There is no separate `wayvid` binary.

#### Scenario: Start playback engine from GUI
- **WHEN** user clicks "Start Engine" button
- **THEN** the GUI initializes the internal playback subsystem
- **AND** updates status to "Engine running"
- **AND** enables wallpaper application functions

#### Scenario: Stop playback engine from GUI
- **WHEN** user clicks "Stop Engine" button
- **THEN** the GUI stops the internal playback subsystem
- **AND** releases video/display resources
- **AND** updates status to "Engine stopped"

### Requirement: GUI-Daemon IPC Communication (Partial Modification)
The GUI application SHALL communicate with external tools (wayvid-ctl) via Unix socket IPC, but internal engine control no longer uses IPC.

#### Scenario: Daemon not running → Engine not running
- **WHEN** the playback engine is stopped
- **THEN** the GUI displays "Engine stopped" status in the sidebar
- **AND** enables the "Start Engine" button
- **AND** IPC server continues running for external control (wayvid-ctl)

## REMOVED Requirements

### Requirement: External Daemon Spawning
**Removed**: The GUI no longer spawns an external `wayvid` process.

**Reason**: v0.5 uses GUI-first architecture where `wayvid-gui` is the main entry point with integrated playback engine.

**Migration**: 
- Users should launch `wayvid-gui --minimized` for background operation
- systemd service updated to use `wayvid-gui --minimized`
- Autostart configs updated accordingly

## ADDED Requirements

### Requirement: Internal Engine State Management
The GUI application SHALL track playback engine state internally without relying on IPC for status updates.

#### Scenario: Engine state initialization
- **WHEN** the GUI starts
- **THEN** the engine is initialized in stopped state
- **AND** user can start engine via UI button

#### Scenario: Engine auto-start on wallpaper apply
- **WHEN** user applies a wallpaper and engine is stopped
- **THEN** the GUI automatically starts the engine
- **AND** then applies the wallpaper
