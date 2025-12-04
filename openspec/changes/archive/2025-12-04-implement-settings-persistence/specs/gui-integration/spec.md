# GUI Integration - Settings Persistence

## ADDED Requirements

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
