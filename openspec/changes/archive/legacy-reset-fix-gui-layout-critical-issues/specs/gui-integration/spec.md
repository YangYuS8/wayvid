## MODIFIED Requirements

### Requirement: Sidebar Navigation
The GUI SHALL provide a sidebar navigation panel with the following characteristics:
- Fixed width of 180px when expanded
- Fixed width of 50px when collapsed
- Text-based labels for navigation items (no Emoji icons)
- Width SHALL NOT change with window resize

#### Scenario: Sidebar expanded state
- **WHEN** the sidebar is in expanded state
- **THEN** sidebar width is exactly 180px regardless of window size

#### Scenario: Sidebar collapsed state
- **WHEN** the sidebar is in collapsed state
- **THEN** sidebar width is exactly 50px regardless of window size

#### Scenario: Window resize stability
- **WHEN** user resizes the application window
- **THEN** sidebar maintains its fixed width

### Requirement: Icon Display Compatibility
The GUI SHALL use text-based labels or ASCII symbols instead of Emoji icons to ensure cross-platform compatibility.

#### Scenario: Navigation icons display
- **WHEN** the sidebar navigation is rendered
- **THEN** all navigation items display readable text labels without garbled characters

#### Scenario: Wallpaper type indicators
- **WHEN** wallpaper cards are displayed in the library
- **THEN** type indicators use text labels (Video, Scene, GIF, Image) instead of Emoji

### Requirement: Thumbnail Fixed Dimensions
The GUI SHALL display wallpaper thumbnails with fixed 1:1 aspect ratio dimensions.

#### Scenario: Thumbnail size consistency
- **WHEN** wallpaper thumbnails are rendered in the grid
- **THEN** each thumbnail has fixed dimensions of 120x120 pixels

#### Scenario: Panel toggle stability
- **WHEN** detail panel is toggled open or closed
- **THEN** thumbnail dimensions remain 120x120 pixels (no size change)

#### Scenario: Thumbnail image fitting
- **WHEN** a wallpaper image is displayed as thumbnail
- **THEN** the image uses Cover content fit to fill the square without distortion
