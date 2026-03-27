# Repository Reset Inventory

## Direction

The repository reset repositions `wayvid` as a Linux dynamic wallpaper platform for users migrating from Wallpaper Engine. We keep only the assets that can accelerate that direction, and we retire or re-evaluate framing that assumes the previous product definition should carry forward unchanged.

## Retained Migration Candidates

- `crates/wayvid-core` as a migration candidate for shared models, configuration, and cross-cutting types.
- `crates/wayvid-engine` as a migration candidate for Linux wallpaper playback, rendering, and runtime integration knowledge.
- `crates/wayvid-library` as a migration candidate for library indexing, metadata handling, and local asset management.
- `crates/wayvid-gui` as a migration candidate for desktop application shell patterns that still fit a library-first Linux experience.
- `crates/wayvid-ctl` as a migration candidate for automation hooks, diagnostics, and operational control surfaces.
- Supporting technical knowledge such as packaging lessons, runtime constraints, and platform integration notes when they help the new Linux dynamic wallpaper platform.

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
