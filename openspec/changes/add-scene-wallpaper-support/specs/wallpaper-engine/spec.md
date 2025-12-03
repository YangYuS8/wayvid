## ADDED Requirements

### Requirement: Scene Wallpaper Parsing
The system SHALL parse Wallpaper Engine scene project files (`project.json` with `type: "scene"`).

#### Scenario: Parse valid scene project
- **WHEN** a scene project.json is loaded
- **THEN** the system extracts scene metadata, layers, and animations
- **AND** creates a renderable scene structure

#### Scenario: Handle unsupported scene features
- **WHEN** a scene contains unsupported features (particles, shaders)
- **THEN** the system logs a warning
- **AND** continues rendering supported elements

### Requirement: Scene Layer Rendering
The system SHALL render scene layers using OpenGL.

#### Scenario: Render image layer
- **WHEN** a scene contains an image layer
- **THEN** the system loads the image resource
- **AND** renders it at the specified position, scale, and rotation

#### Scenario: Render layer hierarchy
- **WHEN** a scene contains multiple layers
- **THEN** layers are rendered in correct z-order
- **AND** blending is applied according to blend mode

### Requirement: Scene Animation
The system SHALL animate scene layers based on animation definitions.

#### Scenario: Scroll animation
- **WHEN** a layer has scroll animation
- **THEN** the layer position updates over time
- **AND** loops according to animation settings

#### Scenario: Scale animation
- **WHEN** a layer has scale animation
- **THEN** the layer size changes over time smoothly

### Requirement: Scene Video Source Type
The system SHALL support scene as a video source type.

#### Scenario: Apply scene wallpaper
- **WHEN** user applies a scene wallpaper to an output
- **THEN** the scene is rendered at the output's resolution
- **AND** animation runs at display refresh rate

## MODIFIED Requirements

### Requirement: Wallpaper Engine Project Detection
The system SHALL detect and categorize Wallpaper Engine project types including video AND scene.

#### Scenario: Detect scene project
- **WHEN** scanning Workshop items
- **THEN** scene projects are identified by `type: "scene"` in project.json
- **AND** marked as compatible if they use supported features

#### Scenario: Display scene type in GUI
- **WHEN** browsing Workshop items
- **THEN** scene wallpapers show "Scene" type badge
- **AND** can be applied like video wallpapers
