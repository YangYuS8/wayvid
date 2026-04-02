# LWE Scene Runtime Support Design

## Summary

Add real desktop support for Wallpaper Engine `scene` projects that are already subscribed on the current machine. This change will extend LWE beyond `video`-only real apply by introducing a scene parsing layer, a typed runtime target for desktop apply, and a scene-capable engine path that can launch supported scene projects on real outputs.

The design intentionally avoids treating `scene` as a disguised video file. Instead, it models scene projects as their own runtime type with their own parsing, compatibility evaluation, and engine session path.

## Goals

- Make subscribed Wallpaper Engine `scene` items truly applicable to the desktop on the current machine.
- Parse enough of a scene project to determine its executable/runtime entrypoint and required assets.
- Replace the current desktop apply assumption that only synced `video` items can be launched.
- Preserve the existing `video` path while adding `scene` as a first-class runtime type.
- Surface clear reasons when a scene project cannot be launched.

## Non-Goals

- Do not implement every possible Wallpaper Engine scene feature in one step.
- Do not add support for `web` runtime playback in this change.
- Do not redesign the entire frontend shell around scene editing or advanced scene controls.
- Do not require a third-party runtime dependency unless later investigation proves it is unavoidable and explicitly approved.
- Do not silently downgrade unsupported scenes into videos and call that “scene support”.

## Current State

The repository already recognizes `scene` as a content type at the metadata level:

- `lwe-library` parses `project.json`
- `WeProject::is_scene()` exists
- compatibility and UI layers can label items as `scene`

However, the real desktop apply path still rejects them. `apps/lwe/src-tauri/src/services/desktop_service.rs` currently resolves the real apply target by insisting the library entry is `WorkshopProjectType::Video`, then extracting a primary video asset path and sending that path into the current engine. The engine path itself is effectively a video/mpv path.

This means scene projects can be recognized and displayed in the app, but not actually applied to the desktop.

## Design Principles

### 1. Treat scene as its own runtime type

The system must not keep pretending a scene project is just a file path. The library/apply boundary needs a typed runtime target so `video` and `scene` can follow different launch paths.

### 2. Parse first, launch second

Desktop apply should not guess from the raw project directory on the fly. Scene project parsing should produce a stable intermediate structure that the app and engine can rely on.

### 3. Keep the frontend thin

Scene support should be implemented primarily in Rust crates and backend services. The frontend should continue to consume compatibility state, Library details, and apply results rather than becoming a format parser.

### 4. Fail explicitly

When a scene cannot be launched, the user should get a precise reason such as missing assets, invalid project structure, or unsupported scene features, rather than a generic “failed to apply” message.

## High-Level Architecture

The feature is split into three layers:

1. `lwe-library`
   Parse Wallpaper Engine scene projects into a typed runtime manifest.

2. `apps/lwe/src-tauri`
   Use the parsed manifest to evaluate compatibility and choose a runtime target for desktop apply.

3. `lwe-engine`
   Add a scene runtime path alongside the existing video/mpv path.

## Runtime Target Model

The current desktop apply code expects a single file path. That is too narrow for scene support.

Introduce a typed runtime target concept at the backend boundary. Conceptually it should distinguish at least:

- `Video { path }`
- `Scene { project_dir, manifest }`

The exact type names can follow existing code style, but the important property is that desktop apply dispatches on target type rather than guessing from `WorkshopProjectType` and returning only a `PathBuf`.

This runtime target should be created from library inspection once and then handed to the engine-launch path.

## Scene Manifest Model

`lwe-library` should produce a scene-focused manifest derived from the project directory and `project.json`. The manifest should contain enough information for later compatibility checks and engine startup, including:

- project directory
- primary scene entry file
- required asset paths resolved relative to the project directory
- preview/cover path when present
- a structured list of unsupported or unknown features encountered during parsing

The manifest does not need to model every internal scene detail. It only needs to represent the minimum information required to decide whether the current runtime can launch the scene and to launch it when possible.

## Parsing Layer

Scene parsing belongs in `lwe-library`, near existing Wallpaper Engine project handling.

The parsing flow should look like this:

1. load `project.json`
2. confirm `type == scene`
3. resolve the declared scene entry file
4. inspect the referenced scene file and its resource graph enough to identify required assets and unsupported features
5. return either:
   - a valid scene manifest, or
   - a structured parse failure

The parser should distinguish at least these failure modes:

- missing scene entry file
- invalid or unreadable scene file
- missing required asset
- unsupported scene construct

## Compatibility Evaluation

Compatibility should stop treating all scenes as a single bucket. Instead, compatibility should reflect the result of real parsing.

Recommended categories:

- supported: manifest parsed and all required features/assets are supported by the current runtime
- degraded or unsupported: scene parsed but depends on unsupported features
- unavailable: scene cannot even be parsed or is missing required files

The exact labels can map onto existing compatibility levels, but the reason text should come from the parsing/runtime analysis rather than generic content-type assumptions.

## Engine Changes

The current engine crate is video-centric and exposes mpv/OpenGL playback through the existing session path. Scene support should be added as a sibling runtime path, not forced through mpv.

That means `lwe-engine` needs a scene-aware session mode. At a minimum, the engine layer should be able to:

- receive a typed scene runtime target
- initialize a scene session for an output
- render or drive that session on the existing output/layer-shell surface model
- surface clear startup/runtime failures back to the app shell

The internal implementation can evolve, but the public engine boundary should no longer imply that every wallpaper is a single media file.

## Desktop Apply Flow

`DesktopService::resolve_real_apply_path()` should be replaced or refactored into a runtime-target resolver.

The new flow should be:

1. inspect the library item
2. build a runtime target:
   - `Video` target for supported video items
   - `Scene` target for supported scene items
3. start or reuse the desktop apply backend
4. dispatch the appropriate engine command for that runtime target
5. wait for apply completion or fail with a clear reason

This preserves the current backend model while making `scene` a first-class launch case.

## Engine Command Boundary

The engine command interface currently implies a file path payload for wallpaper application. That interface should evolve so the command can represent multiple runtime target kinds.

Conceptually, instead of only:

- `ApplyWallpaper { path, output }`

the engine side should accept a payload that can carry either video or scene runtime data.

The exact command naming can follow existing engine conventions, but the command boundary must be typed strongly enough that scene-specific launch data does not get lossy-converted into a path string.

## Failure Handling

Scene runtime support must make failure reasons explicit. The user-facing app result should be able to distinguish:

- scene project missing entry file
- scene project missing required asset
- unsupported scene feature
- engine failed to initialize scene runtime
- engine started but scene apply timed out or failed

These reasons should be preserved through Rust service layers so frontend apply feedback is actionable.

## Real-World Verification Strategy

This feature should be validated against actual subscribed scene projects on the current device, not only synthetic unit fixtures.

Verification should happen at three levels:

### 1. Library parser tests

Use one or more small real or trimmed Wallpaper Engine scene project fixtures to verify manifest parsing, resource resolution, and failure modes.

### 2. Backend service tests

Verify that desktop apply resolves the correct runtime target for:

- supported video item
- supported scene item
- unsupported or broken scene item

### 3. Local manual verification

Use an actual subscribed scene project already present under the local Steam Workshop content directory and confirm:

- the item appears in Library as supported
- apply to monitor succeeds
- Desktop page reflects the assignment
- app restart restores the scene assignment through the unified persistence path

## Fixture Strategy

If licensing and repository size allow, include trimmed fixture projects that preserve the structure needed for parsing. If shipping real assets is not appropriate, include minimal synthetic fixtures that mimic the essential `scene` structure, and keep a documented manual verification path for the real subscribed sample on the current machine.

The parser layer should not depend exclusively on hand-built fake data if a real local sample is available during development.

## Risks and Trade-Offs

### 1. Scene format complexity

Wallpaper Engine scene projects are more complex than videos and may use features the first runtime path cannot support. The design addresses this by requiring explicit parse and compatibility results rather than promising blanket support.

### 2. Engine boundary expansion

Moving from “apply a path” to “apply a runtime target” will touch the app-engine boundary. This is necessary to avoid baking in video-only assumptions even deeper.

### 3. Partial support pressure

There may be temptation to mark scenes as supported after minimal parsing even if rendering support is incomplete. This should be resisted. Real desktop support means the runtime path must actually launch successfully on the machine.

## Success Criteria

This change is complete when:

- LWE can inspect a subscribed `scene` project and build a typed runtime manifest
- compatibility reporting reflects real scene parsing and support status
- desktop apply supports both `video` and `scene` runtime targets
- a supported subscribed `scene` project can be applied to a real monitor on the current machine
- restart restoration can re-apply the saved `scene` assignment through the unified session persistence path
- unsupported or broken scene projects fail with clear reasons instead of generic apply errors
