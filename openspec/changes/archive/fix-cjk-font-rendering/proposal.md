# Fix CJK Font Rendering in GUI

## Summary
Fix Chinese character rendering in the iced-based GUI. Currently, CJK (Chinese/Japanese/Korean) characters display as square boxes (tofu) because the default iced font lacks CJK glyphs.

## Motivation
- GUI i18n implementation is complete with English and Simplified Chinese translations
- Chinese text displays as □□□ (tofu characters) due to missing font glyphs
- Users cannot use the Chinese localization without proper font support

## Solution
Embed a CJK-compatible font (Noto Sans SC) into the application binary and configure iced to use it as the default font.

### Approach: Embedded Font
**Pros:**
- Self-contained binary, works on any system
- No external font dependencies
- Consistent rendering across platforms

**Cons:**
- Increases binary size (~5-10MB for subset font)
- Need to manage font licensing (OFL license is permissive)

### Implementation
1. Download Noto Sans SC font (Google Fonts, OFL licensed)
2. Embed font bytes using `include_bytes!()` macro
3. Configure iced application with `.font()` and `.default_font()`

## Technical Details

### Font Selection
- **Noto Sans SC (Simplified Chinese)**: Good coverage, widely used, OFL licensed
- Subset to common characters if binary size is a concern (future optimization)

### Code Changes
1. `crates/wayvid-gui/fonts/` - Add font file
2. `crates/wayvid-gui/src/app.rs` - Configure font in application builder

### iced API
```rust
iced::application(App::title, App::update, App::view)
    .font(include_bytes!("../fonts/NotoSansSC-Regular.ttf").as_slice())
    .default_font(Font::with_name("Noto Sans SC"))
    // ... other config
```

## Alternatives Considered

### System Font Fallback
- Rely on system-installed CJK fonts
- **Rejected**: Not portable, fails on systems without CJK fonts

### Runtime Font Loading
- Load font from XDG directories at runtime
- **Rejected**: Adds complexity, less reliable

## Impact
- Binary size increase: ~5-10MB
- No API changes
- No breaking changes
- Improves accessibility for Chinese users

## Testing
- Visual verification with Chinese locale
- Test on systems without CJK fonts installed

## Status
- [ ] Download and add font file
- [ ] Modify app.rs to load font
- [ ] Test rendering in Chinese locale
- [ ] Verify binary size impact
