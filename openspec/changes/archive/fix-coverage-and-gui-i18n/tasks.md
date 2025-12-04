## Phase 1: Fix Full Screen Coverage
- [x] 1.1 Change exclusive_zone from 0 to -1 in surface.rs
- [x] 1.2 Add anchor settings for full coverage (all edges)
- [ ] 1.3 Test on noctalia shell with top panel
- [ ] 1.4 Verify no regression on other compositors (Hyprland, Sway)

## Phase 2: GUI Internationalization
- [x] 2.1 Add rust-i18n dependency to Cargo.toml
- [x] 2.2 Create locales directory structure
- [x] 2.3 Extract all UI strings to locale files
- [x] 2.4 Add English (en) locale file
- [x] 2.5 Add Chinese (zh-CN) locale file
- [x] 2.6 Implement language detection (system locale)
- [x] 2.7 Add language switcher in Settings tab

## Phase 3: GUI Modernization
- [x] 3.1 Update color scheme (modern dark theme)
- [x] 3.2 Add card-style layouts for panels
- [x] 3.3 Improve spacing and padding
- [x] 3.4 Add modern icons/emojis consistently
- [x] 3.5 Improve responsive layout for different window sizes
- [x] 3.6 Add loading states and better feedback

## Phase 4: Documentation
- [ ] 4.1 Add documentation style specification to OpenSpec
- [ ] 4.2 Update installation guide in mdbook
- [ ] 4.3 Update configuration reference
- [ ] 4.4 Add i18n section to docs
- [ ] 4.5 Ensure Chinese docs are updated

## Phase 5: Validation
- [ ] 5.1 Test full coverage on multiple compositors
- [ ] 5.2 Test i18n language switching
- [ ] 5.3 Test GUI on various display scales
- [ ] 5.4 Build and test documentation
