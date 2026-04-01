# Repository Reset Inventory

## Direction

The repository reset repositions this repository around `LWE`, a Linux dynamic wallpaper platform for users migrating from Wallpaper Engine. The old `wayvid` name remains only on legacy assets that have not been renamed yet. We keep only the assets that can accelerate that direction, and we retire or re-evaluate framing that assumes the previous product definition should carry forward unchanged.

## Retained Migration Candidates

- `crates/lwe-core` as a migration candidate for shared models, configuration, and cross-cutting types.
- `crates/lwe-engine` as a migration candidate for Linux wallpaper playback, rendering, and runtime integration knowledge.
- `crates/lwe-library` as a migration candidate for library indexing, metadata handling, and local asset management.
- Supporting technical knowledge such as packaging lessons, runtime constraints, and platform integration notes when they help the new Linux dynamic wallpaper platform.

## Retired Legacy Crates

- `crates/wayvid-gui` was a retired legacy GUI shell superseded by the LWE Tauri + Svelte application shell and has now been removed from the repository.
- `crates/wayvid-ctl` was a retired legacy CLI surface that has now been removed from the repository.

Both directories now survive only in git history and archived planning material, and they should not be treated as active product components.

## Retire or Re-Evaluate

- Legacy product positioning that treats the prior `wayvid` application story as the default product frame.
- Release messaging tied to the old GUI-first roadmap instead of the Wallpaper Engine migration narrative.
- Features or docs that imply broad parity before compatibility levels are defined for `video`, `scene`, and later `web` content.
- Workflow assumptions that preserve every existing crate or document without validating relevance to the reset direction.
- User-facing copy, demos, or priorities that do not support Linux migration, wallpaper discovery, import, understanding, or playback.

## Review Criteria

Retain an asset only if it satisfies all of the following review criteria:

1. It contributes directly to the Linux dynamic wallpaper platform direction.
2. It improves Wallpaper Engine migration readiness, compatibility understanding, or daily-use desktop experience.
3. It can be migrated selectively without locking the repository into the retired product framing.
4. Its maintenance cost is justified by near-term platform value for the first-release roadmap.

Assets that fail one or more criteria should be retired, archived for reference, or re-scoped before reuse.
