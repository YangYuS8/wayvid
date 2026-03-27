## MODIFIED Requirements

### Requirement: GUI Layout Structure
The GUI application SHALL use a three-panel layout with a narrow sidebar, main library grid, and right-side detail panel for efficient space utilization.

#### Scenario: Default layout proportions
- **WHEN** the GUI application launches
- **THEN** the sidebar SHALL have a fixed width of 220px
- **AND** the detail panel SHALL have a fixed width of 280px
- **AND** the library grid SHALL fill the remaining space

#### Scenario: Responsive layout on narrow windows
- **WHEN** the window width is less than 800px
- **THEN** the detail panel SHALL be hidden automatically
- **AND** wallpaper details SHALL be shown in a modal or overlay instead

#### Scenario: Collapsible sidebar
- **WHEN** user clicks the sidebar collapse toggle
- **THEN** the sidebar SHALL collapse to icon-only mode (64px)
- **AND** the library grid SHALL expand to use freed space
- **AND** sidebar state SHALL be persisted in settings

### Requirement: Wallpaper Selection and Detail Panel
The GUI application SHALL display detailed information about the selected wallpaper in a dedicated right-side panel.

#### Scenario: Select wallpaper from grid
- **WHEN** user clicks a wallpaper thumbnail in the library grid
- **THEN** the wallpaper SHALL be visually marked as selected
- **AND** the detail panel SHALL show the wallpaper's preview image
- **AND** the detail panel SHALL display title, author, type, and tags

#### Scenario: Detail panel preview image
- **WHEN** a wallpaper is selected
- **THEN** the detail panel SHALL show a large preview (up to 280px wide)
- **AND** animated GIF previews MAY be displayed as static first frame

#### Scenario: Apply wallpaper from detail panel
- **WHEN** user clicks "Apply" button in detail panel
- **THEN** a monitor dropdown SHALL appear if multiple monitors exist
- **AND** the wallpaper SHALL be applied to the selected monitor

#### Scenario: No selection state
- **WHEN** no wallpaper is selected
- **THEN** the detail panel SHALL display a placeholder message
- **AND** suggest user to select a wallpaper from the grid

### Requirement: Library Grid Display
The GUI application SHALL display wallpapers in a responsive grid using Workshop preview images when available.

#### Scenario: Responsive grid columns
- **WHEN** the library panel width changes
- **THEN** the grid SHALL adjust column count automatically
- **AND** maintain consistent thumbnail aspect ratio (16:9)

#### Scenario: Wallpaper card display
- **WHEN** displaying a wallpaper in the grid
- **THEN** the card SHALL show the thumbnail/preview image
- **AND** the card SHALL show the wallpaper title
- **AND** the card SHALL show a type icon (video/scene/gif)

## ADDED Requirements

### Requirement: Workshop Preview Image Loading
The GUI application SHALL prioritize Workshop preview images (preview.jpg, preview.gif) over generated video frame thumbnails.

#### Scenario: Load Workshop preview image
- **WHEN** a Steam Workshop wallpaper is displayed
- **AND** a preview.jpg or preview.gif file exists in the wallpaper folder
- **THEN** the GUI SHALL load the preview file directly as thumbnail
- **AND** SHALL NOT generate a video frame thumbnail

#### Scenario: Preview file detection priority
- **WHEN** scanning a Workshop wallpaper folder
- **THEN** the system SHALL check for files in order: preview.gif, preview.jpg
- **AND** use the first found file as the thumbnail source

#### Scenario: Missing preview fallback
- **WHEN** a Workshop wallpaper has no preview file
- **THEN** the GUI SHALL fall back to video frame generation
- **AND** SHALL use the same thumbnail caching mechanism

#### Scenario: Preview path storage
- **WHEN** a wallpaper with preview file is added to the library
- **THEN** the preview path SHALL be stored in the thumbnail_path field
- **AND** SHALL be persisted in the library database
