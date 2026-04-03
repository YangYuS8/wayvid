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

- deliver a library-first desktop application shell at `src-tauri`
- add Linux desktop integration and daily-use controls
- ship strong `video` and `scene` runtime support
- provide Chinese and English product surfaces

## Active Workspace Decision

- `src-tauri` is the active desktop shell path for current product work
- `wayvid-gui` and `wayvid-ctl` are retired from the active workspace and remain legacy migration references only

## Next Planning Tracks

- `workshop-browsing-and-acquisition`: build the first `LWE` Workshop loop in the active `src-tauri` + `` shell using Rust-owned page snapshots, detail payloads, official Steam handoff actions, and Library projection for synchronized items while treating `wayvid-gui`/`wayvid-ctl` as retired workspace history
- `compatibility-evaluation-and-reporting`: implemented in the active `src-tauri` + `` shell with structured compatibility levels, supporting reasons, and next-step guidance across Workshop and Library surfaces, with follow-on work focused on extending that reporting foundation without promising runtime support beyond reported compatibility
- `desktop-shell-and-library-flow`: the active `LWE` shell now has a locally verified `Library -> Apply to monitor -> Desktop reflects result -> Clear` loop on the current `Wayland + niri` path, backed by real monitor discovery, TOML-backed session persistence under the `lwe` config root, and explicit degraded restore-state handling; follow-on work can deepen runtime coverage and interaction polish rather than establishing the first real desktop action path
- `lwe-usability-pass-v1`: the active shell now distinguishes local `Library` content from the current synced-`Workshop` view more clearly, exposes a stronger Apply entry path, and uses denser detail panels that better support real use; follow-on work can deepen Settings and online Workshop browsing rather than revisiting first-use clarity
- `lwe-settings-mvp`: the active shell now includes editable settings for language, theme, and launch on login, with TOML-backed persistence under the `lwe` config root, visible Steam integration state, graphical-session autostart support, and aligned session or assignment persistence for restored desktop state; follow-on work can extend settings breadth and polish without treating basic settings editing as unfinished groundwork
