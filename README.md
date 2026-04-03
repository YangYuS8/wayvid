# LWE (Repository Reset in Progress)

This repository is being reset from the legacy `wayvid` product into `LWE`, a Linux dynamic wallpaper platform for Wallpaper Engine migration users on Linux.

## Current Status

- The old `wayvid` product story has been retired.
- The repository is keeping only high-value technical assets from the previous codebase.
- The current source of truth for product identity in this worktree is `openspec/config.yaml`.

## Product Direction

The reset points the repository toward a Linux dynamic wallpaper platform that provides:

- in-app Workshop browsing and acquisition orchestration for Wallpaper Engine content
- compatibility visibility before and after import
- first-release focus on `video` and `scene` wallpapers
- recognition of `web` wallpapers for compatibility reporting rather than first-release runtime support
- a polished Linux desktop application experience
- Chinese and English user-facing support

## Naming Direction

- Product name: `LWE`
- Code name and file-path prefix: `lwe`

The old `wayvid` name remains only in historical material preserved for reference.

## What Remains Valuable

- low-level playback and runtime knowledge
- Workshop parsing and import knowledge
- selected shared types and Linux integration code

## Active Product Path

The active LWE product path is now limited to:

- `src-tauri`
- `crates/lwe-core` for shared models and configuration
- `crates/lwe-library` for library and Workshop logic
- `crates/lwe-engine` for runtime and rendering behavior

The legacy crates `crates/wayvid-gui` and `crates/wayvid-ctl` have been removed from the repository. Their history remains available in git history and archived planning material only.

## What Is Changing

- legacy product framing
- top-level docs structure
- future application architecture
- first-release scope definition

## Foundation Checkpoint

The repository reset is considered complete when all of the following are true:

- the top-level product story consistently describes the Linux dynamic wallpaper platform
- the product documentation set points future work at Workshop import, compatibility, and desktop application delivery
- retained legacy assets are called out as technical inputs rather than product commitments
- OpenSpec remains healthy so follow-on plans can start from a clean baseline
- retired OpenSpec change history stays available under `openspec/changes/archive/` without appearing as current reset work

## Documentation Entry Points

- Product overview: `docs/product/overview.md`
- Product roadmap: `docs/product/roadmap.md`
- Documentation reset guide: `docs/README.md`
## AppImage Build

LWE currently supports Linux AppImage bundling through the official Tauri v2 bundler.

Build command:

```bash
cargo tauri build --bundles appimage
```

Current host prerequisite observed on this machine:

- `mksquashfs` must be installed and available on `PATH`

If it is missing, Tauri can complete the `AppDir` staging step but fail on the final `linuxdeploy` AppImage packaging step.
