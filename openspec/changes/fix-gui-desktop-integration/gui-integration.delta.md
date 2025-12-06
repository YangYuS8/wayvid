# gui-integration Spec Delta

## MODIFIED Requirements

### Requirement: Theme Persistence
The GUI application SHALL persist theme settings across application restarts.

#### Scenario: Save theme on toggle
- **WHEN** user clicks the theme toggle button
- **THEN** the GUI updates the internal theme state
- **AND** saves the theme preference ("dark" or "light") to settings file
- **AND** the setting persists after application restart

#### Scenario: Restore theme on startup
- **WHEN** the GUI application starts
- **THEN** it reads the saved theme preference from settings
- **AND** applies the saved theme immediately
- **AND** the UI reflects the saved theme state

## ADDED Requirements

### Requirement: Window Close Handling
The GUI application SHALL handle window close events according to user preferences.

#### Scenario: Minimize to tray enabled
- **WHEN** user closes the window with "minimize to tray" enabled
- **THEN** the GUI minimizes or hides the window
- **AND** continues running in the background
- **AND** the wallpaper playback engine remains active

#### Scenario: Minimize to tray disabled
- **WHEN** user closes the window with "minimize to tray" disabled
- **THEN** the GUI stops all playback
- **AND** the application exits completely

#### Scenario: Wayland compositor limitation
- **WHEN** the Wayland compositor does not support window minimization
- **THEN** the GUI logs a warning message
- **AND** exits gracefully when window is closed
- **AND** displays a notification about the limitation (first time only)

### Requirement: Autostart Documentation
The GUI application SHALL provide clear documentation for autostart configuration.

#### Scenario: XDG autostart (GUI method)
- **WHEN** user enables autostart in GUI settings
- **THEN** a .desktop file is created in `~/.config/autostart/`
- **AND** the file contains `Exec=wayvid-gui --minimized`
- **AND** the application starts automatically on next login (if XDG autostart is supported)

#### Scenario: niri spawn-at-startup
- **WHEN** user reads the documentation for niri configuration
- **THEN** they find instructions for using `spawn-at-startup "wayvid-gui" "--minimized"`
- **AND** the instructions explain the difference from XDG autostart

#### Scenario: Hyprland exec-once
- **WHEN** user reads the documentation for Hyprland configuration
- **THEN** they find instructions for using `exec-once = wayvid-gui --minimized`
- **AND** the instructions include a reference to the example config file
