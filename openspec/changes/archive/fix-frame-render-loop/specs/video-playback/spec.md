## MODIFIED Requirements

### Requirement: Frame Timing and Synchronization
The system SHALL maintain accurate frame timing synchronized with display refresh rate using a dual-callback mechanism.

#### Scenario: mpv update callback triggers render check
- **WHEN** mpv has a new decoded frame available
- **THEN** the mpv update callback SHALL notify the main event loop
- **AND** the main loop SHALL wake up to process the new frame

#### Scenario: Wayland frame callback for vsync
- **WHEN** rendering a frame
- **THEN** a Wayland frame callback SHALL be requested before commit
- **AND** the callback SHALL be used to pace rendering to display vsync
- **AND** rendering SHALL NOT depend solely on frame callbacks

#### Scenario: Continuous rendering without callback dependency
- **WHEN** the main event loop is running
- **THEN** it SHALL poll for new frames using mpv_render_context_update
- **AND** it SHALL render when new frames are available
- **AND** frame callback failures SHALL NOT stop the render loop

#### Scenario: Frame drop handling
- **WHEN** decoding cannot keep up with playback rate
- **THEN** the system SHALL drop frames to maintain sync
- **AND** dropped frames SHALL be logged for diagnostics
