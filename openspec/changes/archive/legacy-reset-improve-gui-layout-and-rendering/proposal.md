# Change: Improve GUI Layout, Rendering Backend, and Library Display

## Why
The current GUI has several usability issues: the sidebar and detail panel have equal widths which is inefficient, the library doesn't leverage Workshop preview images, and the default OpenGL renderer could be improved. These changes will make wayvid's GUI more visually appealing and efficient, closer to the user experience of Wallpaper Engine.

## What Changes
- **Sidebar/Detail Panel Layout**: Redesign sidebar to be narrower (fixed width ~220px), with the main library panel taking remaining space. Wallpaper details will be shown in a right-side panel (like Wallpaper Engine)
- **Vulkan Default Renderer**: Switch from OpenGL to Vulkan as the default renderer for better performance; add setting to switch back to OpenGL
- **Workshop Preview Images**: Use `preview.jpg` or `preview.gif` from Workshop wallpaper folders as thumbnails instead of generating frames
- **Library Panel Redesign**: Adopt Wallpaper Engine-style layout with wallpaper grid on the left, detail panel on the right, and better space utilization

## Impact
- Affected specs: `gui-integration`, `video-playback`
- Affected code:
  - `crates/wayvid-gui/src/views/library.rs` - Layout redesign
  - `crates/wayvid-gui/src/app.rs` - Main layout structure
  - `crates/wayvid-gui/src/settings.rs` - Renderer backend setting
  - `crates/wayvid-gui/src/async_loader.rs` - Preview image loading
  - `crates/wayvid-gui/src/state.rs` - New state for detail panel
  - `crates/wayvid-library/src/thumbnail.rs` - Workshop preview detection
