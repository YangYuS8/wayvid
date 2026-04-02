# LWE Minimal Native Scene Runtime Design

## Summary

Implement the first real native `SceneSession` for LWE so a constrained subset of subscribed Wallpaper Engine `scene` wallpapers can actually render on the desktop. This design builds on the already-added scene parsing, compatibility assessment, runtime-target boundary, and desktop apply wiring, and focuses specifically on replacing the current "scene runtime is not implemented yet" engine stub with a minimal working renderer.

The goal is not blanket Wallpaper Engine scene compatibility. The goal is a small, explicit, real native runtime that can successfully render and restore a supported subset of scene projects on the current machine.

## Goals

- Replace the current explicit scene-runtime stub in `lwe-engine` with a real `SceneSession`.
- Support a clearly defined minimal subset of scene projects.
- Ensure supported scene projects can complete desktop apply and restore on the current machine.
- Fail early and clearly for scene projects outside the supported subset.
- Keep scene runtime setup and failure semantics explicit at session creation time rather than during repeated render-frame errors.

## Non-Goals

- Do not support all Wallpaper Engine scene features in this iteration.
- Do not add `web` wallpaper runtime support.
- Do not add advanced scene controls, editing, or runtime parameter UIs.
- Do not redesign the already-completed runtime-target/desktop apply architecture.
- Do not silently degrade unsupported scenes into some other content type and claim native scene support.

## Current State

The current repository has already completed the upper layers needed for scene support:

- `lwe-library` can parse a minimal `SceneManifest`
- the app shell can assess scene runtime support and build typed runtime targets
- `lwe-engine` now accepts `RuntimeTarget::Scene`
- `desktop_service` applies and restores using typed runtime targets

But the actual engine runtime is still stubbed. `SceneSession::new(...)` returns an explicit unsupported error, so scene projects still cannot render for real.

This means the next unit of work should be engine-focused rather than more app-shell plumbing.

## Supported First Runtime Subset

The first native scene runtime should support only a deliberately small subset of scene projects.

The supported subset is:

- a valid `scene` project with a resolvable scene entry file
- only local bundled assets referenced from that scene project
- no scripting/runtime behavior that requires an embedded scripting engine
- no complex effect graph or feature class that the renderer does not explicitly implement
- enough data to render a stable visible output every frame on the current Wayland/EGL path

For this first iteration, it is acceptable if the subset behaves more like a native “static or lightly animated scene renderer” than a full Wallpaper Engine clone. What matters is that it is genuinely a native scene runtime, not a video fallback.

## Runtime Contract

`SceneSession` must stop being a stub and become a real engine session type with the same lifecycle expectations as the existing video session path:

1. setup
2. first successful render
3. ongoing frame rendering
4. cleanup

Crucially, unsupported scene projects should fail during setup, not after a session has already been accepted into the render loop.

That means:

- scene feature validation must happen before returning a live session
- asset loading required for the supported subset should happen during setup
- if setup succeeds, render should produce real output rather than permanent “not implemented” errors

## Scene Runtime Input Model

The runtime should continue to consume the typed scene runtime target already introduced in the previous scene-support work.

That means `SceneSession` should be constructed from:

- `project_dir`
- `SceneRuntimeManifest`
- `OutputInfo`

No new path-only shortcut should be added.

## Renderer Scope

The first renderer should be as small as possible while still being real.

Recommended first scope:

- load scene asset data required for the supported subset
- upload minimal GPU resources needed to draw it
- render one complete frame through the existing Wayland/EGL surface path
- continue rendering subsequent frames without session-level errors

If animation is included in the supported subset, keep it simple and deterministic. If animation turns out to complicate the first working runtime too much, the first supported subset may be purely static, as long as it is still driven by the native scene runtime path and not by a disguised image/video fallback.

## Session Responsibilities

`SceneSession` should own only scene-runtime concerns:

- validated runtime state for one output
- loaded scene resources needed by the supported subset
- GL/EGL resources needed for rendering
- per-frame draw/update logic

It should not take over responsibilities already handled elsewhere, such as monitor discovery, desktop persistence, or high-level compatibility messaging.

## Setup-Time Validation

A scene should be rejected during setup if any of the following is true:

- the manifest is missing required assets for the supported subset
- the entry scene data is structurally invalid
- the scene uses a feature outside the supported subset
- required GL/runtime resources cannot be initialized

These errors should flow back through the existing engine/app-shell error path so desktop apply feedback stays actionable.

## Rendering Lifecycle

The runtime lifecycle should be:

1. `RuntimeTarget::Scene` enters engine dispatch
2. `SceneSession::new(...)` validates the manifest and creates scene runtime state
3. first render creates or binds any needed EGL/GL resources
4. first successful rendered frame triggers the same apply-success barrier expected by desktop apply
5. later frames continue cleanly
6. cleanup releases scene-specific resources and EGL bindings

This is intentionally parallel to the video session lifecycle so the engine does not grow two completely unrelated models.

## Apply Success Semantics

For scene wallpapers, a desktop apply should only be considered successful once the engine has actually produced the first valid rendered frame for that scene session.

That means the existing apply barrier path should remain frame-based rather than being relaxed to “command accepted.”

If setup fails before first render, the apply must fail.

If setup succeeds but rendering never reaches a first good frame, the apply must also fail instead of being persisted as success.

## Restore Semantics

Startup restore should continue to reuse the same runtime-target and apply path as direct apply.

No separate scene-only restore mechanism should be introduced.

Success criteria for restore are the same as direct apply:

- the scene target resolves successfully
- the scene session initializes successfully
- the first frame is rendered successfully

## Compatibility Alignment

The compatibility layer should be updated so “supported scene” means “supported by this minimal native runtime subset,” not just “manifest parsed.”

That means compatibility and runtime setup must agree on the subset rules. If the runtime cannot actually initialize a scene project, compatibility should not advertise it as supported.

This may require extending scene manifest or scene runtime assessment data so unsupported feature detection is shared rather than duplicated.

## Testing Strategy

This work needs both unit-level and real-sample verification.

### 1. Engine unit tests

Add tests that prove:

- supported minimal scene manifests can create a live `SceneSession`
- unsupported scene features fail during setup
- a successful scene session can report a first-frame success path

### 2. Desktop service integration tests

Add or extend tests proving:

- a supported scene target can complete the desktop apply path
- a supported scene target can complete startup restore
- an unsupported scene target fails before persistence reports success

### 3. Real local verification

Use at least one real subscribed Wallpaper Engine scene project already present on the current machine and verify:

- compatibility shows it as supported
- apply to monitor succeeds
- the desktop visibly changes
- restart restore succeeds

If no current subscribed scene fits the initial supported subset, the work is not complete until either:

- a suitable real local sample is identified, or
- the supported subset is adjusted and re-specified explicitly

## Risks and Trade-Offs

### 1. Scene subset may be too narrow

If the first supported subset is too small, the feature may technically work but not help with real user samples. This is why real local validation is a required success condition.

### 2. Renderer complexity may expand quickly

Scene rendering can become a large project if not tightly scoped. The design avoids this by forcing setup-time gating and a narrow supported subset.

### 3. Compatibility/runtime drift

If compatibility says “supported” but runtime setup still fails, users lose trust. The implementation must share enough validation logic to avoid this split-brain behavior.

## Success Criteria

This change is complete when:

- `SceneSession::new(...)` can successfully initialize a real supported scene session
- supported scenes no longer fail with “scene runtime is not implemented yet”
- desktop apply treats a scene as successful only after first real rendered frame
- startup restore can reapply a supported scene through the same runtime path
- unsupported scenes fail during setup with clear reasons
- at least one real subscribed scene project on the current machine is successfully applied and restored end-to-end
