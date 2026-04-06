# Agent Guide

This file is intended for coding agents and contributors who need repository-specific operational context.

## Project snapshot

- Product: `LWE`
- Platform: Linux desktop app (Tauri + Svelte)
- Primary user goal: migrate practical Wallpaper Engine workflows to Linux

## Scope priorities

Current first-release runtime focus:

- Video wallpapers

Current non-first-release runtime targets:

- Scene wallpapers (private format reverse engineering cost is currently too high)
- Web wallpapers (compatibility recognition/reporting only)

## Packaging and release model

- Stable release: GitHub release artifacts + AUR `lwe`
- Prerelease: GitHub prerelease artifacts + AUR `lwe-git`
- Linux artifacts: `.deb`, `.rpm`, `.AppImage`

## Key paths

- Frontend app: `src/`
- Tauri app: `src-tauri/`
- Core/library/engine crates: `crates/lwe-core`, `crates/lwe-library`, `crates/lwe-engine`
- AUR packaging: `packaging/aur/lwe`, `packaging/aur/lwe-git`
- Workflows: `.github/workflows/`

## Versioning notes

- Workspace version source: `Cargo.toml`
- Stable package follows workspace semver
- Prerelease version is derived in Actions (`<base>-beta.<run_number>+<short_sha>`)

## Repository housekeeping

- Keep user-facing docs concise in root README files
- Keep agent-focused operational detail in `docs/agent/`
