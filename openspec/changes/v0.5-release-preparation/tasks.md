# v0.5 Release Preparation - Tasks

## Task 1: Desktop File Integration Verification
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] Review `packaging/wayvid-gui.desktop` contents
2. [ ] Run `scripts/install.sh --user`
3. [ ] Check `~/.local/share/applications/wayvid-gui.desktop` exists
4. [ ] Verify icon path is correct
5. [ ] Launch from application menu (e.g., rofi, wofi, application-menu)
6. [ ] Verify window appears with correct title

### Acceptance Criteria
- Desktop file installed to correct location
- Application launches from menu
- Icon displays correctly (if icon exists)

---

## Task 2: Installation Script Testing
**Status**: Not Started
**Priority**: High

### Steps
1. [ ] Clean any existing installation: `scripts/uninstall.sh --user`
2. [ ] Build release: `cargo build --release --workspace`
3. [ ] Run install: `scripts/install.sh --user`
4. [ ] Verify binaries exist:
   - `~/.local/bin/wayvid-gui`
   - `~/.local/bin/wayvid-ctl`
5. [ ] Run installed binary: `~/.local/bin/wayvid-gui`
6. [ ] Run uninstall: `scripts/uninstall.sh --user`
7. [ ] Verify cleanup complete

### Acceptance Criteria
- Install script completes without errors
- All binaries are executable
- Uninstall removes all installed files

---

## Task 3: Release Documentation
**Status**: Not Started
**Priority**: Medium

### Files to Update
1. [ ] `CHANGELOG.md` - Add v0.5.0 section with:
   - GUI-first architecture
   - iced 0.13 framework
   - i18n support (en, zh-CN)
   - Workshop integration
   - Installation scripts
2. [ ] `README.md` - Verify accuracy of:
   - Installation instructions
   - Usage instructions
   - Feature list
3. [ ] `docs/` - Quick review for outdated info

### Acceptance Criteria
- CHANGELOG has complete v0.5.0 entry
- README reflects current functionality

---

## Task 4: Git Operations for Release
**Status**: Not Started
**Priority**: Medium

### Steps
1. [ ] Stage all changes: `git add -A`
2. [ ] Commit with message: `feat: v0.5 GUI-first architecture complete`
3. [ ] Push to origin: `git push origin v0.5-gui-first`
4. [ ] Create PR to main branch
5. [ ] After merge, tag release: `git tag v0.5.0-alpha.1`
6. [ ] Push tag: `git push origin v0.5.0-alpha.1`

### Acceptance Criteria
- All changes committed
- PR created and ready for review
- Tag created after merge

---

## Progress Tracking

| Task | Status | Blockers |
|------|--------|----------|
| Desktop Integration | Not Started | None |
| Install Script Test | Not Started | None |
| Documentation | Not Started | None |
| Git Operations | Not Started | Depends on above |
| **IPC Server Integration** | **Completed** | None |
| **Layer Surface Fix** | **Completed** | None |
| **Hot-swap Optimization** | **Completed** | None |

---

## Completed Tasks (Previous Sessions)

### IPC Server Integration for wayvid-ctl
**Status**: ✅ Completed (2025-12-06)

#### Changes Made
1. Added `command_sender()` method to `EngineHandle` to expose the calloop channel sender
2. Added `CommandSender` re-export in `wayvid-engine/src/lib.rs`
3. Integrated `IpcServer` into `EngineController`:
   - Added `ipc_server: Option<IpcServer>` field
   - Start IPC server automatically when engine starts
   - Stop IPC server when engine stops

#### Verification
- `wayvid-ctl status` ✅ Works
- `wayvid-ctl ping` ✅ Works
- `wayvid-ctl apply <path> --output <name>` ✅ Works
- `wayvid-ctl stop --output <name>` ✅ Works

### Layer Surface Full Screen Fix
**Status**: ✅ Completed (2025-12-06)

#### Issue
Wallpaper was not covering the top bar (noctalia-shell)

#### Fix
Changed `layer_surface.set_exclusive_zone(0)` to `layer_surface.set_exclusive_zone(-1)` to ignore other layer surfaces' exclusive zones

### Hot-swap Wallpaper Optimization
**Status**: ✅ Completed (2025-12-06)

#### Issue
Screen flicker when switching wallpapers due to layer surface recreation

#### Fix
1. Added `load_new_wallpaper()` method to `WallpaperSession` that uses MPV's `loadfile` command
2. Modified `apply_wallpaper_to_output()` to reuse existing layer surfaces instead of destroying/recreating
3. Only clear/swap buffers when MPV has a valid frame to display

---

## Notes for Next Session
- Start with Task 1 (Desktop Integration) as it's a quick verification
- Task 2 (Install Script) should be tested on a clean state
- Task 3 and 4 can be done after verification tasks pass
- Consider implementing actual output status query for `wayvid-ctl status` (currently returns empty list)
