# Fix Full Screen Coverage and GUI Improvements

## Why

### 1. Wallpaper Coverage Issue
The wallpaper doesn't cover the full screen area on systems with panels (like noctalia shell's top bar). This happens because:
- Current `set_exclusive_zone(0)` only means "don't reserve space"
- It doesn't instruct the compositor to ignore OTHER windows' exclusive zones
- Need `set_exclusive_zone(-1)` to request full coverage ignoring all exclusive zones

### 2. GUI Usability Issues
- Interface looks dated with default egui styling
- No internationalization support (all strings hardcoded in English)
- Chinese users cannot use the GUI effectively
- No modern UI elements (cards, better spacing, icons)

### 3. Documentation Gap
- Docs may not reflect latest features
- No clear documentation style guidelines in OpenSpec
- Need concise, user-focused documentation

## What

### Phase 1: Fix Full Screen Coverage
- Change `set_exclusive_zone(0)` to `set_exclusive_zone(-1)`
- This tells compositor to place wallpaper behind all other layers
- Wallpaper will now cover entire screen including panel areas

### Phase 2: GUI Modernization & i18n
- Integrate `rust-i18n` crate for internationalization
- Add Chinese (zh-CN) and English (en) language packs
- Modernize UI with better styling:
  - Rounded corners and cards
  - Better color scheme (dark/light mode aware)
  - Improved spacing and typography
  - Modern iconography

### Phase 3: Documentation Update
- Add documentation style guide to OpenSpec
- Update mdbook docs with concise style
- Ensure all new features are documented

## Impact

**Benefits:**
- Full screen wallpaper coverage on all compositors
- Chinese-speaking users can use GUI natively
- Modern, professional-looking interface
- Better documentation for all users

**Risks:**
- i18n adds compile-time dependency
- UI changes require testing on multiple displays
- Need to maintain translations going forward

**Effort:** ~6-8 hours total
- Phase 1: 30 minutes (simple fix)
- Phase 2: 4-6 hours (i18n + UI refactor)
- Phase 3: 1-2 hours (docs update)
