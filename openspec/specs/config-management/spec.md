# Configuration Management

## Purpose
The configuration system provides flexible, pattern-based configuration for video wallpapers across multiple monitors with hot-reload support and validation.

## Requirements

### Requirement: YAML Configuration Format
The system SHALL use YAML format for configuration files with clear validation and error reporting.

#### Scenario: Valid configuration parsing
- **WHEN** a valid `config.yaml` file is loaded
- **THEN** the configuration SHALL be parsed successfully
- **AND** all settings SHALL be applied to the running daemon

#### Scenario: Invalid YAML syntax
- **WHEN** a configuration file contains invalid YAML syntax
- **THEN** the daemon SHALL fail to start with a clear error message
- **AND** the error message SHALL indicate the line number and syntax issue

#### Scenario: Missing required fields
- **WHEN** a configuration file omits required fields
- **THEN** the daemon SHALL fail to start with a descriptive error
- **AND** the error SHALL list which required fields are missing

### Requirement: Output Pattern Matching
The system SHALL support pattern-based output configuration to match monitors by name, model, or serial number.

#### Scenario: Exact name match
- **WHEN** an output name exactly matches a configuration pattern
- **THEN** that configuration SHALL be applied to the output
- **AND** other patterns SHALL not be considered

#### Scenario: Wildcard pattern matching
- **WHEN** a configuration uses wildcard patterns (e.g., `HDMI-*`)
- **THEN** all matching outputs SHALL use that configuration
- **AND** wildcards SHALL support both `*` (any characters) and `?` (single character)

#### Scenario: Default fallback configuration
- **WHEN** an output does not match any specific pattern
- **THEN** the `default` configuration SHALL be applied
- **AND** the system SHALL log which default is being used

#### Scenario: Priority-based matching
- **WHEN** multiple patterns match the same output
- **THEN** the most specific pattern SHALL take precedence
- **AND** exact matches SHALL override wildcards
- **AND** longer patterns SHALL override shorter patterns

### Requirement: Per-Output Video Sources
The system SHALL allow independent video sources for each monitor.

#### Scenario: Different videos per monitor
- **WHEN** configuration specifies different video files for each output
- **THEN** each monitor SHALL display its assigned video
- **AND** videos SHALL play independently

#### Scenario: Video source types
- **WHEN** a video source is configured
- **THEN** it SHALL support file paths (single video)
- **AND** it SHALL support directory paths (random selection)
- **AND** it SHALL support Steam Workshop IDs

### Requirement: Video Scaling Modes
The system SHALL support multiple video scaling modes to fit different aspect ratios.

#### Scenario: Fill mode
- **WHEN** scaling mode is set to `fill`
- **THEN** video SHALL be scaled to cover entire display
- **AND** aspect ratio SHALL be preserved by cropping if necessary

#### Scenario: Contain mode
- **WHEN** scaling mode is set to `contain`
- **THEN** video SHALL be scaled to fit within display
- **AND** aspect ratio SHALL be preserved with letterboxing if necessary

#### Scenario: Stretch mode
- **WHEN** scaling mode is set to `stretch`
- **THEN** video SHALL be scaled to fill display exactly
- **AND** aspect ratio MAY be distorted to match display dimensions

### Requirement: Hot Configuration Reload
The system SHALL detect and apply configuration changes without restarting.

#### Scenario: Configuration file modification
- **WHEN** the configuration file is modified while daemon is running
- **THEN** the system SHALL detect the change within 1 second
- **AND** the new configuration SHALL be applied automatically
- **AND** video playback SHALL transition smoothly to new settings

#### Scenario: Invalid hot reload
- **WHEN** a configuration file is modified with invalid content
- **THEN** the system SHALL log an error
- **AND** the previous valid configuration SHALL remain active
- **AND** the daemon SHALL not crash

### Requirement: Configuration Validation
The system SHALL validate all configuration parameters before applying them.

#### Scenario: Video file existence check
- **WHEN** a configuration references a video file path
- **THEN** the system SHALL verify the file exists
- **AND** an error SHALL be raised if the file is missing or inaccessible

#### Scenario: Supported file format validation
- **WHEN** a video file is specified
- **THEN** the system SHALL check the file extension
- **AND** a warning SHALL be logged for unsupported formats

#### Scenario: Numeric range validation
- **WHEN** configuration contains numeric parameters (e.g., FPS, volume)
- **THEN** values SHALL be validated against allowed ranges
- **AND** out-of-range values SHALL cause configuration load failure

### Requirement: Configuration File Locations
The system SHALL search for configuration files in standard XDG directories.

#### Scenario: Primary configuration location
- **WHEN** the daemon starts
- **THEN** it SHALL first check `~/.config/wayvid/config.yaml`
- **AND** this location SHALL take precedence over others

#### Scenario: System-wide configuration
- **WHEN** no user configuration exists
- **THEN** the daemon SHALL check `/etc/wayvid/config.yaml`
- **AND** system configuration SHALL serve as fallback

#### Scenario: Explicit configuration path
- **WHEN** a configuration path is specified via CLI argument
- **THEN** that path SHALL be used exclusively
- **AND** standard locations SHALL not be checked

### Requirement: Example Configuration Generation
The system SHALL provide example configuration files for common use cases.

#### Scenario: Generate example config command
- **WHEN** user runs `wayvid-ctl config example`
- **THEN** an example configuration SHALL be written to stdout
- **AND** the example SHALL include comments explaining each option

#### Scenario: Install example config
- **WHEN** no configuration file exists at startup
- **THEN** the system SHALL offer to create an example configuration
- **AND** the example SHALL be installed to `~/.config/wayvid/config.example.yaml`
