# GUI Integration - Async Thumbnail Loading

## ADDED Requirements

### Requirement: Asynchronous Thumbnail Loading
The GUI application SHALL load wallpaper thumbnails asynchronously without blocking the main UI thread.

#### Scenario: Load thumbnail for visible wallpaper
- **WHEN** a wallpaper card becomes visible in the library view
- **THEN** the GUI requests thumbnail generation in a background task
- **AND** displays a loading placeholder until complete
- **AND** replaces placeholder with actual thumbnail when ready

#### Scenario: Workshop wallpaper preview
- **WHEN** a Steam Workshop wallpaper has a preview image (preview.jpg)
- **THEN** the GUI loads the preview image directly
- **AND** does not generate a video frame thumbnail

#### Scenario: Video thumbnail generation
- **WHEN** a local video wallpaper needs a thumbnail
- **AND** no cached thumbnail exists
- **THEN** the GUI extracts a frame from the video using ThumbnailGenerator
- **AND** caches the result to disk for future use

#### Scenario: Thumbnail loading failure
- **WHEN** thumbnail generation fails (corrupted file, unsupported format)
- **THEN** the GUI displays a fallback icon based on wallpaper type
- **AND** does not retry failed thumbnails repeatedly

### Requirement: Thumbnail Caching
The GUI application SHALL cache generated thumbnails to improve startup performance.

#### Scenario: Disk cache lookup
- **WHEN** a thumbnail is requested
- **THEN** the GUI first checks `~/.cache/wayvid/thumbnails/` for cached version
- **AND** uses cached version if file exists and is valid

#### Scenario: Cache persistence
- **WHEN** a thumbnail is generated
- **THEN** the GUI saves it to disk cache as WebP format
- **AND** the cached thumbnail survives application restarts

#### Scenario: Memory cache limit
- **WHEN** the in-memory thumbnail cache exceeds 100 entries
- **THEN** the GUI evicts least-recently-used thumbnails from memory
- **AND** keeps disk cache intact for future loads

### Requirement: Visibility-Based Loading
The GUI application SHALL prioritize thumbnail loading for visible wallpapers.

#### Scenario: Scroll-based loading
- **WHEN** user scrolls through the library
- **THEN** the GUI requests thumbnails for newly visible wallpapers
- **AND** cancels pending requests for wallpapers scrolled out of view

#### Scenario: Large library performance
- **WHEN** library contains 1000+ wallpapers
- **THEN** the GUI maintains smooth scrolling (60fps)
- **AND** only loads thumbnails for visible items plus buffer
