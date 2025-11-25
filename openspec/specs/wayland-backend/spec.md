# Wayland Backend

## Purpose
The Wayland backend provides compositor integration via wlr-layer-shell protocol, handling output management, surface creation, and compositor-specific optimizations.

## Requirements

### Requirement: wlr-layer-shell Protocol Support
The system SHALL use wlr-layer-shell-unstable-v1 protocol for background layer rendering.

#### Scenario: Layer surface creation
- **WHEN** a new output is detected
- **THEN** a layer surface SHALL be created for the background layer
- **AND** the surface SHALL be anchored to fill the entire output
- **AND** the layer SHALL be set to bottom (below windows)

#### Scenario: Exclusive zone configuration
- **WHEN** creating a layer surface
- **THEN** exclusive zone SHALL be set to 0
- **AND** other windows SHALL not be restricted by the wallpaper

#### Scenario: Keyboard interactivity disabled
- **WHEN** configuring layer surface
- **THEN** keyboard interactivity SHALL be set to none
- **AND** the wallpaper SHALL not capture keyboard input

### Requirement: Output Hotplug Support
The system SHALL dynamically handle monitor connection and disconnection events.

#### Scenario: New monitor connected
- **WHEN** a new monitor is connected while daemon is running
- **THEN** the system SHALL detect the new output within 1 second
- **AND** a layer surface SHALL be created for the new output
- **AND** video playback SHALL start automatically based on configuration

#### Scenario: Monitor disconnected
- **WHEN** a monitor is disconnected
- **THEN** the system SHALL detect the removal
- **AND** the associated layer surface SHALL be destroyed
- **AND** video decoding SHALL stop for that output
- **AND** resources SHALL be released

#### Scenario: Monitor reconnection
- **WHEN** a previously configured monitor is reconnected
- **THEN** the system SHALL restore the previous configuration
- **AND** video playback SHALL resume automatically

### Requirement: Multi-Output Management
The system SHALL manage multiple displays simultaneously with independent configurations.

#### Scenario: Independent video playback
- **WHEN** multiple monitors are active
- **THEN** each SHALL have its own layer surface
- **AND** each SHALL display its configured video source
- **AND** playback SHALL be synchronized to each display's refresh rate

#### Scenario: Output information retrieval
- **WHEN** querying output information
- **THEN** the system SHALL provide output name, model, manufacturer, and serial
- **AND** this information SHALL be used for pattern matching

### Requirement: Fractional Scaling Support
The system SHALL support HiDPI displays with fractional scaling.

#### Scenario: Fractional scale factor application
- **WHEN** an output has fractional scaling enabled (e.g., 1.5x)
- **THEN** the layer surface SHALL be rendered at the scaled resolution
- **AND** video SHALL be upscaled smoothly without aliasing
- **AND** performance SHALL not degrade significantly

#### Scenario: Scale factor change
- **WHEN** output scale factor is changed dynamically
- **THEN** the layer surface SHALL be resized accordingly
- **AND** video rendering SHALL adapt to new dimensions
- **AND** no visual glitches SHALL occur during transition

### Requirement: Compositor-Specific Optimizations
The system SHALL detect and apply compositor-specific optimizations where available.

#### Scenario: Niri workspace awareness
- **WHEN** running under Niri compositor
- **THEN** the system SHALL detect when wallpaper is not visible in any workspace
- **AND** video decoding SHALL pause to save resources
- **AND** playback SHALL resume when wallpaper becomes visible again

#### Scenario: Hyprland layer management
- **WHEN** running under Hyprland
- **THEN** standard wlr-layer-shell SHALL be used
- **AND** no special optimizations SHALL be applied (works out-of-box)

#### Scenario: Generic Wayland compositor
- **WHEN** running under unknown compositor
- **THEN** the system SHALL use standard wlr-layer-shell protocol
- **AND** a warning SHALL be logged about potential compatibility issues
- **AND** basic functionality SHALL still work

### Requirement: Surface Damage Tracking
The system SHALL efficiently update only changed regions of the display.

#### Scenario: Full-frame damage on video update
- **WHEN** a new video frame is ready
- **THEN** the entire surface SHALL be marked as damaged
- **AND** the compositor SHALL request a redraw

#### Scenario: No damage when paused
- **WHEN** video playback is paused
- **THEN** no damage SHALL be reported to compositor
- **AND** the surface SHALL remain static without unnecessary redraws

### Requirement: EGL Context Integration
The system SHALL create and manage EGL context for OpenGL rendering.

#### Scenario: EGL context creation
- **WHEN** a layer surface is created
- **THEN** an EGL context SHALL be initialized with the Wayland surface
- **AND** the context SHALL support OpenGL ES 3.0 minimum
- **AND** DMA-BUF import extension SHALL be checked and enabled if available

#### Scenario: EGL surface resize
- **WHEN** output resolution or scale changes
- **THEN** the EGL surface SHALL be resized to match
- **AND** video rendering SHALL adapt to new dimensions
- **AND** aspect ratio calculations SHALL be updated

#### Scenario: Context cleanup on error
- **WHEN** EGL context creation fails
- **THEN** partial resources SHALL be cleaned up
- **AND** an error SHALL be reported with diagnostic information
- **AND** other outputs SHALL continue functioning

### Requirement: Wayland Event Loop Integration
The system SHALL integrate with Wayland event loop for responsive compositor communication.

#### Scenario: Event dispatch timing
- **WHEN** Wayland events are pending
- **THEN** they SHALL be dispatched within the main event loop
- **AND** event processing SHALL not block video rendering
- **AND** frame timing SHALL remain accurate

#### Scenario: Registry event handling
- **WHEN** Wayland registry announces new globals
- **THEN** relevant protocols SHALL be bound automatically
- **AND** output information SHALL be updated

### Requirement: Graceful Degradation
The system SHALL handle protocol extension unavailability gracefully.

#### Scenario: Missing wlr-layer-shell protocol
- **WHEN** compositor does not support wlr-layer-shell
- **THEN** the daemon SHALL fail to start with a clear error message
- **AND** the error SHALL suggest compatible compositors

#### Scenario: Missing fractional scale protocol
- **WHEN** compositor does not support fractional scaling protocol
- **THEN** the system SHALL fall back to integer scaling
- **AND** a warning SHALL be logged about potential HiDPI issues
- **AND** basic functionality SHALL continue working
