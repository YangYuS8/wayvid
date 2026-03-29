# AppImage Packaging Notes

This directory documents retired AppImage packaging work for the legacy `wayvid-gui` and `wayvid-ctl` binaries.

## Current Status

- The active product path is `apps/lwe/src-tauri`.
- No current repository policy says AppImage is a supported deliverable for the active LWE shell.
- The files in this directory remain as legacy packaging notes and migration reference only.

## How To Read These Files Now

- `build-appimage.sh` is a retirement stub that explains the AppImage flow is no longer active.
- `test-appimage.sh` is a retirement stub for the same legacy path.
- `AppRun`, desktop metadata, and icons remain only to preserve historical packaging context.

## What Changed

Previous docs in this directory described AppImage output as a normal release artifact for `wayvid-gui` and `wayvid-ctl`.
That is no longer true. Those binaries are retired legacy references, and the repository does not currently ship a replacement AppImage flow for `lwe-app-shell`.

## If You Need A Current Verification Path

Use the active workspace checks from the repository root:

```bash
cargo metadata --no-deps
cargo test -p lwe-app-shell
```

Any future AppImage work for LWE should start as a new packaging proposal instead of reusing these retired legacy instructions as if they were current release guidance.
