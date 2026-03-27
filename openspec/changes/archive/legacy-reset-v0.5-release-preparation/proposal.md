# v0.5 Release Preparation

## Summary
Finalize the v0.5 GUI-first release by completing remaining tasks: desktop file integration verification, installation script testing, and release tagging.

## Current Status (as of 2024-12-04)

### Completed Work
1. **Workspace restructuring** - 5 crates: wayvid-core, wayvid-engine, wayvid-library, wayvid-gui, wayvid-ctl
2. **GUI framework migration** - iced 0.13 with CJK font support (Noto Sans CJK SC)
3. **i18n implementation** - rust-i18n 3.x with English and Simplified Chinese
4. **Warning cleanup** - All 80 warnings resolved to 0
5. **iced 0.13 scrollable panic fix** - Fixed `center(Length::Fill)` issue in scrollable content
6. **Installation scripts** - Created `scripts/install.sh` and `scripts/uninstall.sh`
7. **Workshop integration UI** - State, messages, and library view updated
8. **Daemon control UI** - Sidebar controls for start/stop daemon

### Key Technical Decisions
- Used `center_x(Length::Fill)` + `center_y(Length::Fixed(...))` instead of `center(Length::Fill)` in scrollable content
- Added `#![allow(dead_code)]` to modules with reserved functionality
- Explicit lifetime annotations (`Element<'_, Message>`) for view functions

## Remaining Tasks

### Task 1: Desktop File Integration Verification
**Priority: High**

Files involved:
- `packaging/wayvid-gui.desktop` - Already exists
- `scripts/install.sh` - Installs desktop file

Verify:
- [ ] Desktop file has correct `Exec=` path after installation
- [ ] Icon is properly referenced
- [ ] Application appears in launcher after install
- [ ] `StartupWMClass` matches window class

### Task 2: Installation Script Testing
**Priority: High**

Test matrix:
- [ ] `scripts/install.sh --user` - User mode installation
- [ ] `scripts/uninstall.sh --user` - User mode uninstallation
- [ ] Binary symlinks work correctly
- [ ] Desktop file is in correct XDG path

### Task 3: Release Documentation
**Priority: Medium**

- [ ] Update CHANGELOG.md with v0.5.0 changes
- [ ] Review README.md for accuracy
- [ ] Verify installation instructions

### Task 4: Git Operations for Release
**Priority: Medium**

- [ ] Commit all changes on `v0.5-gui-first` branch
- [ ] Merge to main branch
- [ ] Create `v0.5.0-alpha.1` tag

## Technical Context for Next Session

### Branch Information
- **Current branch**: `v0.5-gui-first`
- **Target branch**: `main`

### Build Commands
```bash
# Build all crates
cargo build --workspace

# Build GUI only (zero warnings expected)
cargo build -p wayvid-gui

# Run GUI
cargo run -p wayvid-gui

# Run tests (78 tests)
cargo test --workspace
```

### Project Structure
```
wayvid/
├── crates/
│   ├── wayvid-core/     # Shared types, config, layout
│   ├── wayvid-engine/   # Wayland backend, video rendering
│   ├── wayvid-library/  # Database, scanner, workshop
│   ├── wayvid-gui/      # iced GUI application
│   └── wayvid-ctl/      # CLI tool
├── scripts/
│   ├── install.sh       # Installation script
│   └── uninstall.sh     # Uninstallation script
├── packaging/
│   └── wayvid-gui.desktop
└── openspec/
    └── changes/
        └── v0.5-release-preparation/
```

### Key Files Modified Recently
- `crates/wayvid-gui/src/app.rs` - Font config, sidebar, view routing
- `crates/wayvid-gui/src/views/library.rs` - Source filters, wallpaper grid
- `crates/wayvid-gui/src/state.rs` - Workshop state, source filter enum
- `crates/wayvid-gui/src/messages.rs` - Workshop and daemon messages
- `crates/wayvid-library/src/database.rs` - dead_code fix
- `crates/wayvid-library/src/workshop.rs` - mut warning fix

### Available Tools
The next session should have access to:
1. **Serena** - Code analysis and editing (preferred for Rust code)
2. **Context7** - Library documentation lookup
3. **Sequential Thinking** - Complex planning tasks
4. **Playwright** - Browser automation (if needed for testing)

### Conventions
1. **Language**: Code in English, PR/issues in Simplified Chinese
2. **Encoding**: UTF-8 without BOM
3. **Formatting**: Run `cargo fmt` before committing
4. **Linting**: `cargo clippy` should pass with no errors

## Non-Goals
- Online Workshop browsing (future release)
- Config file modification for compositors (use example configs)
- Vulkan backend completion (separate proposal)

## Success Criteria
- GUI launches without errors or warnings
- Installation script creates working installation
- Desktop file appears in application launcher
- Wallpapers can be browsed and applied (mocked or real)
