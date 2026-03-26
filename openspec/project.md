# Project Context

## Purpose

This repository is being reset from the legacy `wayvid` direction into a new Linux dynamic wallpaper platform.

The product exists to help Wallpaper Engine migration users on Linux discover, acquire, import, understand, and run Workshop wallpapers in a polished desktop application.

## Product Priorities

- Desktop application first
- Workshop-centered differentiation
- video + scene first-release focus
- compatibility visibility as a product feature
- bilingual-first user experience (Chinese and English)

## Repository Strategy

- Keep only high-value technical assets.
- Treat old docs, naming, and architecture assumptions as removable unless proven useful.
- Build follow-on changes from the approved product blueprint in `docs/superpowers/specs/2026-03-27-linux-dynamic-wallpaper-platform-design.md`.

## Technical Starting Point

Current Rust crates are migration candidates:

- `crates/wayvid-engine` for low-level playback and Wayland runtime knowledge
- `crates/wayvid-library` for Workshop parsing/import knowledge and library mechanics
- `crates/wayvid-core` for reusable shared domain types only where still relevant

Current GUI/application structure is not assumed to be the foundation for the new product.

## Planning Rule

Do not treat the first approved blueprint as a single implementation unit. Split future work into separate plans for repository reset, Workshop loop, compatibility, runtime, and application shell.
