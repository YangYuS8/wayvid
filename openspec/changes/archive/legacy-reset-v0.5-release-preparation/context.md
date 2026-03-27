# Session Handoff Context

This document provides complete context for continuing work on the wayvid v0.5 release.

## Project Overview

**wayvid** is a video wallpaper application for Wayland compositors (niri, Hyprland, Sway). Version 0.5 introduces a GUI-first architecture using the iced framework.

### Repository
- **Location**: `/home/yangyus8/Code/wayvid`
- **Branch**: `v0.5-gui-first`
- **Owner**: YangYuS8

## Current State

### Build Status
- **Warnings**: 0 (all 80 warnings cleaned)
- **Tests**: 78 passing
- **GUI**: Launches successfully with Chinese locale

### Workspace Structure
```
crates/
├── wayvid-core/      # Shared types, config parsing, layout
├── wayvid-engine/    # Wayland backend, MPV integration
├── wayvid-library/   # SQLite database, folder scanner, workshop
├── wayvid-gui/       # iced 0.13 GUI application
└── wayvid-ctl/       # CLI control tool
```

### Key Dependencies
- `iced` 0.13 - GUI framework
- `rust-i18n` 3.x - Internationalization
- `sys-locale` 0.3 - System locale detection
- `tokio` - Async runtime
- `rusqlite` - Database

## Completed Work (This Session)

### 1. CJK Font Rendering Fix
- Embedded `Noto Sans CJK SC` font in `crates/wayvid-gui/fonts/`
- Configured iced with `.font()` and `.default_font()`
- Chinese characters now render correctly

### 2. iced 0.13 Scrollable Panic Fix
**Root Cause**: Using `center(Length::Fill)` inside scrollable content causes panic:
> "scrollable content must not fill its vertical scrolling axis"

**Solution**: Replace with `center_x(Length::Fill)` + `center_y(Length::Fixed(...))` or use `center_x()` only.

**Files Fixed**:
- `views/library.rs` - Wallpaper card thumbnail container
- `views/folders.rs` - Empty state container
- `views/about.rs` - Removed `Space::with_height(Length::Fill)`
- `views/settings.rs` - Removed `Space::with_height(Length::Fill)`
- `views/monitors.rs` - Layout fixes
- `widgets/thumbnail_image.rs`
- `widgets/wallpaper_card.rs`
- `widgets/monitor_selector.rs`
- `app.rs` - Sidebar layout

### 3. Warning Cleanup
All 80 warnings resolved:
- **Lifetime elision warnings** (10): Added explicit `Element<'_, Message>` return types
- **Unused imports** (5): Removed `warn`, `Watcher`, `Alignment`, etc.
- **Dead code** (~65): Added `#![allow(dead_code)]` to modules with reserved functionality

**Modules marked with `#![allow(dead_code)]`**:
- `async_loader.rs` - Async thumbnail loading (future)
- `service.rs` - Background service management (future)
- `ipc.rs` - Daemon communication (future)
- `settings.rs` - Settings persistence (future)
- `widgets/thumbnail_image.rs`
- `widgets/wallpaper_card.rs`

### 4. Installation Scripts
Created `scripts/install.sh` and `scripts/uninstall.sh` with:
- `--user` mode (default): Install to `~/.local/`
- `--system` mode: Install to `/usr/local/`
- Desktop file installation
- Systemd user service (optional)

### 5. Workshop Integration UI
- Added `SourceFilter` enum (All, Workshop, Local)
- Added source filter tabs in library header
- Added Workshop scanning state and messages
- Added daemon control buttons in sidebar

## Remaining Tasks

See `openspec/changes/v0.5-release-preparation/tasks.md` for detailed task list.

### High Priority
1. **Desktop File Integration Verification** - Test that GUI appears in app launchers
2. **Installation Script Testing** - End-to-end install/uninstall test

### Medium Priority
3. **Release Documentation** - Update CHANGELOG.md and README.md
4. **Git Operations** - Commit, merge to main, tag v0.5.0-alpha.1

## Important Commands

```bash
# Build (zero warnings expected)
cargo build -p wayvid-gui

# Run GUI
cargo run -p wayvid-gui

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Check lints
cargo clippy --workspace

# Install (user mode)
./scripts/install.sh --user

# Uninstall (user mode)
./scripts/uninstall.sh --user
```

## Configuration Files

### i18n Locales
- `crates/wayvid-gui/locales/en.toml` - English
- `crates/wayvid-gui/locales/zh-CN.toml` - Simplified Chinese

### Desktop File
- `packaging/wayvid-gui.desktop`

### Example Configs
- `configs/config.example.yaml`
- `configs/hyprland-autostart.conf`
- `configs/niri-autostart.kdl`

## Coding Conventions

### General
- Language: Code in English, comments can be English or Chinese
- Encoding: UTF-8 without BOM
- Formatting: `cargo fmt` before commit
- Linting: `cargo clippy` must pass

### Rust Style
- Explicit lifetimes in view functions: `fn view(state: &State) -> Element<'_, Message>`
- Use `#[allow(dead_code)]` for reserved functionality
- Prefer `center_x()` over `center()` in scrollable content

### MCP Services Priority
1. **Serena** - Local code analysis and editing (preferred)
2. **Context7** - Library documentation lookup
3. **Sequential Thinking** - Complex multi-step planning

## Known Issues

### iced 0.13 Quirks
- `center(Length::Fill)` panics in scrollable content
- Use `center_x(Fill)` + explicit height instead

### Security Audit
- `cargo-audit` shows 2 unmaintained warnings (instant, paste)
- These are transitive dependencies from iced/wgpu, not security vulnerabilities

## Contact Points

### Files Most Likely to Need Changes
- `crates/wayvid-gui/src/app.rs` - Main application logic
- `crates/wayvid-gui/src/views/*.rs` - UI views
- `crates/wayvid-gui/src/state.rs` - Application state
- `scripts/install.sh` - Installation script

### OpenSpec Location
- `openspec/changes/v0.5-release-preparation/` - Current active proposal
- `openspec/changes/archive/` - Completed proposals
