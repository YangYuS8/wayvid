## ADDED Requirements

### Requirement: Render Backend Selection
The system SHALL allow users to select their preferred rendering backend (OpenGL, Vulkan, or Auto) through the GUI settings page.

#### Scenario: User selects Vulkan backend
- **WHEN** user opens Settings tab
- **AND** user selects "Vulkan" from the renderer dropdown
- **THEN** the selection is saved to the configuration file
- **AND** a message indicates that restart is required for changes to take effect

#### Scenario: Vulkan unavailable warning
- **WHEN** user attempts to select Vulkan backend
- **AND** Vulkan is not available on the system
- **THEN** a warning message is displayed explaining Vulkan is unavailable
- **AND** the selection falls back to OpenGL

### Requirement: Custom Wallpaper Folders
The system SHALL allow users to add custom folders for wallpaper scanning instead of importing individual files.

#### Scenario: Add wallpaper folder
- **WHEN** user clicks "Add Wallpaper Folder" button
- **AND** user selects a folder path
- **THEN** the folder is added to the watched folders list
- **AND** all video files in the folder (recursively) are added to the library

#### Scenario: Folder path persistence
- **WHEN** user adds a wallpaper folder
- **THEN** the folder path is saved and persisted across sessions
- **AND** the folder is automatically scanned on next GUI launch

#### Scenario: Remove wallpaper folder
- **WHEN** user clicks remove button next to a folder
- **THEN** the folder is removed from watched folders
- **AND** wallpapers from that folder are removed from the library
