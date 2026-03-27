## 1. GUI Layout Redesign

### 1.1 Sidebar Optimization
- [x] 1.1.1 Reduce sidebar width to fixed 180px (changed from 220px)
- [x] 1.1.2 ~~Make sidebar collapsible~~ (Removed - user requested fixed sidebar)
- [x] 1.1.3 Update sidebar icons and spacing for narrower layout
- [x] 1.1.4 Move monitor selector to sidebar bottom

### 1.2 Library Panel Redesign
- [ ] 1.2.1 Implement resizable split view (library grid | detail panel)
- [x] 1.2.2 Create wallpaper grid view with responsive columns (using row().wrap())
- [x] 1.2.3 Implement wallpaper detail panel on the right side (320px width)
- [x] 1.2.4 Add wallpaper selection state management
- [x] 1.2.5 Display selected wallpaper preview, title, author, tags in detail panel
- [x] 1.2.6 Add "Apply Wallpaper" button in detail panel
- [x] 1.2.7 Add detail panel toggle button in header

### 1.3 Responsive Layout
- [x] 1.3.1 Auto-adjust grid columns based on available width
- [ ] 1.3.2 Hide detail panel on narrow windows (< 800px)
- [x] 1.3.3 Preserve detail_panel_visible state in settings

## 2. Rendering Backend Configuration

### 2.1 Vulkan Default
- [x] 2.1.1 Set Vulkan as default renderer in settings (default: "vulkan")
- [ ] 2.1.2 Add fallback to OpenGL if Vulkan unavailable
- [ ] 2.1.3 Log renderer backend on startup

### 2.2 Settings Integration
- [x] 2.2.1 Add renderer backend option to settings struct (GuiSettings.renderer)
- [ ] 2.2.2 Add renderer dropdown in Settings view
- [x] 2.2.3 Message handler for renderer change (ChangeRenderer message)
- [x] 2.2.4 Persist renderer preference to settings file

## 3. Workshop Preview Image Support

### 3.1 Preview Detection
- [x] 3.1.1 Scan for preview image path from project.json
- [x] 3.1.2 Prefer preview image over generated video thumbnail
- [x] 3.1.3 Update WallpaperItem to store thumbnail_path

### 3.2 Thumbnail Loading
- [x] 3.2.1 Load Workshop preview images directly (skip generation)
- [ ] 3.2.2 Support animated GIF previews (display as static or animated)
- [x] 3.2.3 Fall back to video frame generation if preview missing

### 3.3 Scanner Integration
- [x] 3.3.1 Update Workshop scanner to detect preview files
- [x] 3.3.2 Store thumbnail_path in database
- [x] 3.3.3 Database schema includes thumbnail_path column

## 4. Testing and Polish

### 4.1 Testing
- [x] 4.1.1 Test layout on various window sizes
- [ ] 4.1.2 Verify Vulkan/OpenGL switching works correctly
- [x] 4.1.3 Validate Workshop preview loading
- [ ] 4.1.4 Performance test with large library (1000+ items)

### 4.2 Documentation
- [ ] 4.2.1 Update user guide with new layout description
- [ ] 4.2.2 Document renderer setting in reference

## 5. Additional Improvements (Added during implementation)

### 5.1 About Page
- [x] 5.1.1 Replace emoji with SVG logo
- [x] 5.1.2 Add svg feature to iced dependencies

### 5.2 Card Display
- [x] 5.2.1 Remove type labels from cards (cleaner look)
- [x] 5.2.2 Fixed 120x120 square thumbnails
- [x] 5.2.3 UTF-8 safe string truncation for wallpaper names
