# LWE Static Real Scene Content Design

## Summary

Replace the current placeholder solid-color scene renderer with the first real scene-content renderer for LWE. The first target is not animation; it is to display real static content originating from the scene payload itself, using the existing package-backed runtime model.

The core rule is: the rendered output must come from the scene’s actual content, not from a synthetic color and not from the external preview/cover image.

## Goals

- Replace the solid-color placeholder renderer in `SceneSession`.
- Render real scene content for a narrow, explicit subset of scene packages.
- Keep the package-backed runtime identity model introduced in the previous phase.
- Ensure at least one real subscribed local scene project renders recognizably as scene content instead of a solid color.
- Preserve clear unsupported behavior for scenes whose static content cannot yet be derived honestly.

## Non-Goals

- Do not implement general animation playback in this phase.
- Do not implement a complete `scene.pkg` entry enumerator.
- Do not render the preview/cover image as a substitute for scene content.
- Do not claim support for arbitrary scene packages beyond the explicitly supported static subset.
- Do not undo the package-backed runtime identity model.

## Current State

The codebase now has:

- package-backed `SceneManifest` resolution using validated top-level `scene.pkg`
- package-backed compatibility and library/runtime-target selection
- package-backed engine/apply/status identity
- desktop apply and restore aligned with that package-backed identity

But the renderer still produces only a deterministic clear color. That proves the runtime path works, but it does not satisfy the product goal of showing actual scene content.

## Design Principle

The next renderer must remain honest to current evidence.

That means:

- render real scene-derived content only when we can justify how it was obtained
- reject unsupported packages clearly when we cannot derive real static content safely
- do not hide uncertainty behind fake rendering, preview images, or guessed package internals

## Supported First Static Subset

This phase should support only scene packages where a real static content source can be obtained from the current package-backed understanding.

The supported subset is:

- scene project resolves to a validated `scene.pkg`
- package variant is one of the known supported `PKGV00xx` forms already accepted by the manifest/runtime contract
- package-backed content path yields a real static image or equivalent renderable asset that can be justified from current evidence
- the renderer can draw that content natively through the current EGL/OpenGL path

If the scene package does not expose a static renderable asset through evidence-backed means, it must remain unsupported in this phase.

## Content Source Rule

The renderer may only use a content source that is part of the scene payload itself.

Allowed examples:

- a real image-like asset discovered through a validated package-backed rule
- a package-backed static scene payload that the engine can draw directly

Forbidden examples:

- the workshop preview image from top-level `preview.*`
- a synthetic solid color
- a guessed internal package file selected without evidence

## Runtime Model

The external runtime contract remains unchanged:

- `SceneManifest` stays package-backed
- `RuntimeTarget::Scene` stays package-backed
- apply barrier and status identity stay package-backed

The change happens inside `SceneSession`: instead of clearing to a color, it should load and draw the first real static content source supported by the current subset.

## Content Derivation Strategy

This phase needs a narrow, explicit content-derivation rule.

The derivation logic should:

1. start from the validated `scene.pkg` payload
2. use only package facts already justified by current investigation or by this phase’s new, narrowly evidenced extension
3. resolve one real static content source
4. expose that source to the renderer as a concrete scene-content input

If this cannot be done for a given scene package, the package must fail subset validation rather than falling back to a fake renderer.

## Renderer Responsibilities

The first real static renderer should own only what is needed to show the derived static content:

- load the derived content source into GPU resources
- draw it onto the layer-shell surface
- report first-frame success only after that real content is rendered

It should not pretend to support full scene graphs, animation systems, scripts, or generalized package extraction.

## First-Frame Success Semantics

A scene apply may only succeed once the real static scene-derived content has been rendered to the output.

That means:

- first-frame success must no longer be tied to “render loop ran” while drawing a synthetic clear color
- unsupported or unresolved scene packages must fail before that success signal

## Compatibility Alignment

Compatibility must be tightened to the new static-content subset.

That means a scene should be marked supported only when:

- it resolves to a valid package-backed runtime payload
- the runtime can derive a real static content source from that payload

If the package-backed payload is valid but no supported content source can yet be derived, compatibility must say unsupported or partially supported rather than “ready.”

## Testing Strategy

### 1. Engine unit tests

Add tests proving:

- unsupported scene packages still fail clearly
- supported static-content scenes no longer use the placeholder color-only path
- first-frame success is tied to real content rendering

### 2. Backend service tests

Add tests proving:

- compatibility only marks scenes supported when real static content can be derived
- desktop apply success for scenes depends on the static-content renderer path, not the placeholder path
- restore reuses the same static-content scene path

### 3. Real local validation

At least one subscribed local scene sample must be re-tested and should show:

- visible static scene content on the desktop
- no solid-color fallback
- successful restore showing the same scene-derived static content

## Stop Condition

If implementing real static content requires inventing generalized package enumeration or other internal structure not supported by the current evidence, stop and create a new package-internals design rather than guessing.

This phase is only valid if it can derive real static content honestly from the package-backed model.

## Risks and Trade-Offs

### 1. Static subset may still be small

The first phase may support only a small fraction of scene packages. That is acceptable if the supported cases are real and clearly defined.

### 2. Hidden package complexity

Even a static-content path may hit internal package structure we cannot yet justify. In that case the correct action is to stop and design the next parser layer, not to fake support.

### 3. Compatibility churn

Compatibility may temporarily narrow as we move from placeholder “it runs” behavior to honest “it shows real content” behavior. That is expected and correct.

## Success Criteria

This phase is complete when:

- `SceneSession` no longer renders a solid-color placeholder for supported scenes
- the renderer draws real static content derived from the scene payload itself
- first-frame success depends on that real static rendering path
- compatibility only marks scenes supported when that static-content path is available
- at least one real subscribed local scene project renders recognizable static scene content and restores correctly
