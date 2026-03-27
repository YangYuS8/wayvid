## ADDED Requirements

### Requirement: Configurable Rendering Backend
The system SHALL support switching between Vulkan and OpenGL rendering backends, with Vulkan as the default.

#### Scenario: Default to Vulkan renderer
- **WHEN** the GUI application starts without explicit renderer configuration
- **THEN** the application SHALL attempt to use Vulkan (wgpu Vulkan backend) for rendering
- **AND** SHALL log "Using Vulkan renderer" at startup

#### Scenario: Vulkan unavailable fallback
- **WHEN** the GUI application starts with Vulkan configured
- **AND** Vulkan is not available on the system
- **THEN** the application SHALL automatically fall back to OpenGL
- **AND** SHALL log a warning about the fallback
- **AND** SHALL update the settings to reflect actual renderer used

#### Scenario: User selects OpenGL renderer
- **WHEN** user changes renderer setting to OpenGL in Settings view
- **THEN** the application SHALL save the preference to settings file
- **AND** SHALL display a notification that restart is required
- **AND** SHALL use OpenGL renderer after restart

#### Scenario: Renderer setting persistence
- **WHEN** renderer preference is changed
- **THEN** the setting SHALL be persisted to `~/.config/wayvid/settings.yaml`
- **AND** SHALL be respected on subsequent application launches

#### Scenario: Display current renderer in Settings
- **WHEN** user opens Settings view
- **THEN** the current active renderer SHALL be displayed
- **AND** a dropdown SHALL allow selection between Vulkan and OpenGL
- **AND** the current selection SHALL be highlighted
