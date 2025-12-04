# implement-async-thumbnail-loading - Tasks

## 1. Enable and Integrate async_loader Module
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 1.1 Remove `#![allow(dead_code)]` from `async_loader.rs`
2. [ ] 1.2 Export `AsyncLoader`, `ThumbnailRequest`, `ThumbnailResult` from module
3. [ ] 1.3 Add `async_loader: AsyncLoader` to `AppState`
4. [ ] 1.4 Initialize `AsyncLoader` in `AppState::new()`
5. [ ] 1.5 Ensure thumbnail cache directory is created on startup

### Acceptance Criteria
- async_loader module compiles without dead_code warning
- AsyncLoader is initialized and accessible in app state

---

## 2. Add Thumbnail Messages
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 2.1 Add `Message::RequestThumbnails(Vec<ThumbnailRequest>)` 
2. [ ] 2.2 Add `Message::ThumbnailLoaded(String, Vec<u8>)` - already exists
3. [ ] 2.3 Add `Message::ThumbnailFailed(String, String)` for error handling
4. [ ] 2.4 Add `Message::ThumbnailBatchComplete(usize)` for batch tracking
5. [ ] 2.5 Update message handlers in `app.rs`

### Acceptance Criteria
- All thumbnail-related messages are defined
- Message handlers update state appropriately

---

## 3. Implement Thumbnail Subscription
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 3.1 Create `thumbnail_subscription()` in async_loader.rs (already exists, verify)
2. [ ] 3.2 Track pending thumbnail requests in AppState
3. [ ] 3.3 Integrate subscription in `App::subscription()`
4. [ ] 3.4 Handle subscription events to update state
5. [ ] 3.5 Implement request deduplication to avoid double-loading

### Acceptance Criteria
- Subscription processes thumbnail requests asynchronously
- Loaded thumbnails are stored in state.thumbnails HashMap

---

## 4. Update Library View for Thumbnail Display
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] 4.1 Import `iced::widget::Image` in library.rs
2. [ ] 4.2 Create `ThumbnailState` enum: NotLoaded, Loading, Loaded(Handle), Failed
3. [ ] 4.3 Update `wallpaper_card()` to check thumbnail state
4. [ ] 4.4 Display `Image::new(handle)` for loaded thumbnails
5. [ ] 4.5 Display loading spinner for Loading state
6. [ ] 4.6 Display fallback icon for Failed state
7. [ ] 4.7 Request thumbnail if NotLoaded and visible

### Acceptance Criteria
- Wallpaper cards display actual thumbnails when available
- Loading and error states are visually indicated

---

## 5. Implement Visibility-Based Loading
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 5.1 Track scroll position in library view
2. [ ] 5.2 Calculate visible wallpaper range based on scroll
3. [ ] 5.3 Request thumbnails only for visible + buffer items
4. [ ] 5.4 Cancel pending requests for items scrolled out of view
5. [ ] 5.5 Implement `VirtualScroll` helper from async_loader.rs

### Acceptance Criteria
- Only visible wallpapers trigger thumbnail loading
- Scrolling doesn't cause excessive thumbnail requests

---

## 6. Thumbnail Cache Management
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 6.1 Implement disk cache using `~/.cache/wayvid/thumbnails/`
2. [ ] 6.2 Hash wallpaper path/id for cache filename
3. [ ] 6.3 Check disk cache before generating thumbnail
4. [ ] 6.4 Save generated thumbnails to disk cache
5. [ ] 6.5 Implement memory cache size limit (e.g., 100 thumbnails)
6. [ ] 6.6 Implement LRU eviction for memory cache

### Acceptance Criteria
- Thumbnails persist across app restarts
- Memory usage stays bounded with large libraries

---

## 7. Workshop Preview Image Loading
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 7.1 Check if wallpaper has `thumbnail_path` set
2. [ ] 7.2 For Workshop items, load preview.jpg directly (no generation)
3. [ ] 7.3 Fall back to video frame extraction if preview missing
4. [ ] 7.4 Handle various image formats (jpg, png, webp)

### Acceptance Criteria
- Workshop wallpapers show their preview images
- Preview loading is faster than video frame extraction

---

## 8. Testing and Validation
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] 8.1 Test with large Workshop library (1000+ wallpapers)
2. [ ] 8.2 Verify smooth scrolling performance
3. [ ] 8.3 Verify thumbnail cache works across restarts
4. [ ] 8.4 Test error handling for corrupted/missing files
5. [ ] 8.5 Verify memory usage stays reasonable

### Acceptance Criteria
- GUI remains responsive with large libraries
- No memory leaks or excessive disk usage

---

## Progress Tracking

| Task | Status | Blockers |
|------|--------|----------|
| Enable async_loader | Not Started | None |
| Thumbnail Messages | Not Started | Task 1 |
| Subscription Integration | Not Started | Task 1, 2 |
| Library View Update | Not Started | Task 2, 3 |
| Visibility-Based Loading | Not Started | Task 4 |
| Cache Management | Not Started | Task 3 |
| Workshop Preview Loading | Not Started | Task 4 |
| Testing | Not Started | All above |

## Technical Notes

### iced Image Widget
```rust
use iced::widget::Image;
use iced::widget::image::Handle;

// From bytes
let handle = Handle::from_bytes(thumbnail_data);
Image::new(handle).width(Length::Fill).height(Length::Fixed(120.0))
```

### Thumbnail Generation
`wayvid_library::ThumbnailGenerator` extracts video frames:
```rust
let generator = ThumbnailGenerator::with_size(256, 144);
let result = generator.generate(&video_path)?;
// result.data contains WebP image bytes
```

### Cache Path Hashing
```rust
use std::hash::{Hash, Hasher};
fn get_cache_path(cache_dir: &Path, id: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    cache_dir.join(format!("{:x}.webp", hasher.finish()))
}
```
