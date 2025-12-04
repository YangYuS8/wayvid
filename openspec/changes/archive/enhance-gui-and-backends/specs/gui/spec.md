## ADDED Requirements

### Requirement: Workshop Preview Images
The system SHALL display preview images (gif/jpg) for Steam Workshop wallpapers in the GUI library grid.

#### Scenario: Display workshop preview
- **WHEN** GUI displays a Steam Workshop wallpaper entry
- **AND** a preview.gif or preview.jpg exists in the wallpaper folder
- **THEN** the preview image is displayed as the wallpaper thumbnail
- **AND** animated GIFs show animation in the thumbnail

#### Scenario: Preview image fallback
- **WHEN** GUI displays a wallpaper without a preview image
- **THEN** a default placeholder image is displayed
- **AND** the wallpaper title is shown as fallback text

### Requirement: Scene Wallpaper Support in GUI
The system SHALL provide GUI support for configuring and managing scene-type wallpapers.

#### Scenario: Scene wallpaper indicator
- **WHEN** GUI displays a scene-type wallpaper
- **THEN** a visual indicator (badge/icon) shows it is a scene wallpaper
- **AND** the indicator distinguishes it from video wallpapers

#### Scenario: Apply scene wallpaper
- **WHEN** user clicks on a scene wallpaper
- **THEN** the scene wallpaper is applied to the selected monitor
- **AND** scene layers are properly loaded and rendered

#### Scenario: Scene wallpaper preview
- **WHEN** user hovers over a scene wallpaper
- **THEN** a preview of the scene is shown
- **AND** the preview shows the base layer image

### Requirement: Settings Tab Enhancement
The system SHALL provide an enhanced settings page with categorized options.

#### Scenario: Display renderer settings
- **WHEN** user opens Settings tab
- **THEN** a "Renderer" section is displayed
- **AND** it contains backend selection dropdown (Auto/OpenGL/Vulkan)
- **AND** it shows current GPU/driver information

#### Scenario: Display folder management
- **WHEN** user opens Settings tab
- **THEN** a "Wallpaper Folders" section is displayed
- **AND** it lists all configured wallpaper source folders
- **AND** it provides add/remove buttons for folder management
