# wayland-backend Specification (Delta)

## MODIFIED Requirements

### Requirement: wlr-layer-shell Protocol Support
**Clarification**: Layer surfaces are now managed by embedded engine within GUI process

The system SHALL use wlr-layer-shell-unstable-v1 protocol for background layer rendering.

#### Scenario: Layer surface creation
- **WHEN** a new output is detected by the engine
- **THEN** a layer surface SHALL be created using the engine's dedicated Wayland connection
- **AND** the surface SHALL be anchored to fill the entire output
- **AND** the layer SHALL be set to bottom (below windows)

### Requirement: Wayland Event Loop Integration
**Modified**: Engine has its own event loop separate from iced

~~The system SHALL integrate with Wayland event loop for responsive compositor communication.~~

The engine SHALL run its own Wayland event loop in a dedicated thread, separate from the GUI's iced event loop.

#### Scenario: Dual connection model
- **WHEN** the GUI starts with embedded engine
- **THEN** the GUI uses iced's internal Wayland connection for UI rendering
- **AND** the engine creates a separate Wayland connection for layer surfaces
- **AND** both connections operate independently without interference

#### Scenario: Event dispatch timing
- **WHEN** Wayland events are pending on the engine's connection
- **THEN** they SHALL be dispatched within the engine's event loop
- **AND** event processing SHALL not block GUI rendering
- **AND** frame timing SHALL remain accurate

## ADDED Requirements

### Requirement: Independent Wayland Connection
The embedded engine SHALL maintain its own Wayland connection independent of the GUI framework.

#### Scenario: Connection initialization
- **WHEN** the engine starts
- **THEN** it SHALL establish a new connection to the Wayland compositor
- **AND** bind required protocols (wl_compositor, wl_shm, zwlr_layer_shell_v1)
- **AND** create an event queue for processing events

#### Scenario: Connection cleanup
- **WHEN** the engine stops
- **THEN** all layer surfaces SHALL be destroyed
- **AND** the Wayland connection SHALL be closed
- **AND** no resources SHALL leak

#### Scenario: Connection failure
- **WHEN** the engine cannot connect to Wayland
- **THEN** an error SHALL be reported to the GUI
- **AND** the GUI SHALL display appropriate error message
- **AND** the GUI itself SHALL continue functioning
