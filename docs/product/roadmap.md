# Product Roadmap

## Phase 1: Repository Reset

- rewrite top-level product metadata around the Linux dynamic wallpaper platform
- replace legacy docs with the Wallpaper Engine migration narrative
- identify retained technical assets for future migration work

## Phase 2: Workshop and Compatibility Foundation

- build Workshop browsing and acquisition orchestration
- recognize wallpaper content types including `video`, `scene`, and `web` for import and compatibility reporting
- define compatibility levels and explanations for Wallpaper Engine content

## Phase 3: First-Release Application

- deliver a library-first desktop application shell at `apps/lwe/src-tauri`
- add Linux desktop integration and daily-use controls
- ship strong `video` and `scene` runtime support
- provide Chinese and English product surfaces

## Active Workspace Decision

- `apps/lwe/src-tauri` is the active desktop shell path for current product work
- `wayvid-gui` and `wayvid-ctl` are retired from the active workspace and remain legacy migration references only

## Next Planning Tracks

- `workshop-browsing-and-acquisition`: build the first `LWE` Workshop loop in the active `apps/lwe/src-tauri` + `apps/lwe` shell using Rust-owned page snapshots, detail payloads, official Steam handoff actions, and Library projection for synchronized items while treating `wayvid-gui`/`wayvid-ctl` as retired workspace history
- `compatibility-evaluation-and-reporting`: implemented in the active `apps/lwe/src-tauri` + `apps/lwe` shell with structured compatibility levels, supporting reasons, and next-step guidance for imported Wallpaper Engine content, without promising runtime support beyond reported compatibility
- `desktop-shell-and-library-flow`: define the application shell plan for library management, playback controls, and Linux desktop integration
