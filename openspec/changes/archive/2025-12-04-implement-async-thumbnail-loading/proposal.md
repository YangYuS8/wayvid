# Change: Implement Async Thumbnail Loading

## Why

The wayvid-gui currently shows placeholder icons (🖼️, 📷, or type icons) instead of actual wallpaper thumbnails. The `async_loader.rs` module is fully implemented but marked as `dead_code` and not integrated with the library view. This results in:

- Poor user experience - users can't visually identify wallpapers
- Workshop wallpapers have preview images that aren't displayed
- The thumbnail generation infrastructure exists but isn't used
- Memory inefficient - no caching or lazy loading

With thousands of wallpapers in a typical Workshop library, efficient async thumbnail loading is essential for a responsive GUI.

## What Changes

### Core Integration
- **Enable async_loader.rs**: Remove `#![allow(dead_code)]` and integrate the module
- **Subscription System**: Connect `thumbnail_subscription()` to `App::subscription()`
- **Cache Integration**: Use `AsyncLoader` for managing thumbnail cache

### Library View Updates
- **Thumbnail Display**: Replace placeholder text with actual `iced::widget::Image`
- **Loading States**: Show loading spinner while thumbnail is being generated
- **Error Handling**: Show fallback icon on thumbnail generation failure

### Performance Optimization
- **Virtual Scrolling**: Implement `VirtualScroll` helper for large libraries
- **Batch Loading**: Request thumbnails in batches for visible items only
- **Memory Management**: Limit cache size and implement LRU eviction

## Impact

### Affected Specs
- None (new capability)

### Affected Code
- `crates/wayvid-gui/src/async_loader.rs` - Enable and extend
- `crates/wayvid-gui/src/views/library.rs` - Integrate thumbnail display
- `crates/wayvid-gui/src/app.rs` - Add subscription and message handlers
- `crates/wayvid-gui/src/state.rs` - Add loading state tracking
- `crates/wayvid-gui/src/messages.rs` - Add thumbnail messages

### Dependencies
- `wayvid_library::ThumbnailGenerator` - Already implemented
- `iced::widget::Image` - For displaying thumbnails

## Success Criteria

1. Workshop wallpapers display their preview images
2. Local video wallpapers show generated thumbnail frames
3. Thumbnails load asynchronously without blocking UI
4. Scrolling through large libraries remains smooth
5. Thumbnail cache persists across sessions
