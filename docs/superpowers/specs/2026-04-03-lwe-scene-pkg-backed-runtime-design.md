# LWE Scene Package-Backed Runtime Design

## Summary

Redefine LWE scene runtime support so it is built around `scene.pkg` as the real runtime payload artifact for inspected Wallpaper Engine scene projects, rather than around a guessed top-level `scene.json` file. This design uses the completed `scene.pkg` investigation to replace the old entrypoint assumption and establish a more honest runtime boundary for the next implementation phase.

The key shift is simple: for the inspected local scene samples, `project.json.file = "scene.json"` is not a reliable top-level filesystem path. The runtime must therefore treat `scene.pkg` as the externally visible scene payload and defer any deeper internal package semantics until the codebase has evidence to support them.

## Goals

- Make the next scene runtime iteration consume `scene.pkg` as the primary runtime payload for supported scene projects.
- Remove the last remaining architectural dependence on top-level `scene.json` as a runtime input.
- Keep the runtime design honest about what is known versus unknown about package internals.
- Preserve clear failure behavior for unsupported or insufficiently understood package variants.
- Allow future deeper `scene.pkg` parsing without forcing the current runtime architecture to guess package internals.

## Non-Goals

- Do not implement a general `scene.pkg` entry enumerator in this design.
- Do not assume internal package layout beyond the evidence currently gathered.
- Do not promise support for all `PKGV00xx` variants.
- Do not reintroduce the old top-level `scene.json`-based manifest/runtime model.
- Do not claim full Wallpaper Engine scene compatibility.

## Established Facts

From the completed local investigation:

- multiple real local scene projects declare `project.json.file = "scene.json"`
- those same project directories contain `scene.pkg`, not a top-level `scene.json`
- `scene.pkg` samples show `PKGV0021` / `PKGV0023` headers
- `strings` output exposes scene-related names such as `scene.json`, material paths, shader paths, and texture/resource paths
- current evidence supports a narrow package signature/version sniff and a package-backed manifest fallback
- current evidence does not support stable package entry enumeration or a reliable internal index model

These facts mean the next runtime should be package-backed, not guessed-entry-backed.

## Problem Reframing

The previous runtime effort tried to answer: “How do we render `scene.json`?”

That is now the wrong question.

The correct question is: “How do we define a minimal native runtime that accepts a known-valid `scene.pkg` payload, while staying honest about the fact that internal package structure is still only partially understood?”

## Runtime Input Model

The scene runtime input should be shifted from a guessed entry-file model to a package-backed model.

For the next implementation phase, the scene runtime should consume a package-backed scene target conceptually equivalent to:

- project directory
- package path (`scene.pkg`)
- package signature/version facts
- any limited package-backed metadata already justified by evidence

The package path becomes the authoritative external payload reference.

If `scene.json` is later proven to be a stable packaged entry with reliable enumeration support, that can extend the model. It should not be assumed now.

## Manifest Model Shift

`SceneManifest` should be treated as a package-backed scene descriptor, not as proof of a top-level entry file.

The manifest should preserve at least:

- the declared logical entry hint from `project.json` when present
- the concrete package-backed payload path used for runtime input
- provenance describing whether a runtime path is directly declared or inferred from the investigated `scene.pkg` rule

The manifest should not imply that package-backed resolution equals internal package understanding.

## Runtime Contract

The next scene runtime iteration should take a `scene.pkg` payload as input and be explicit about what stage it is in:

1. package accepted
2. package-backed runtime setup started
3. first native frame succeeded or setup failed

This allows the runtime to make honest progress without pretending it already understands the full internal package graph.

## Supported Runtime Subset

The supported subset for the next runtime phase should be narrowed again around what the code can honestly consume:

- scene project resolves to a `scene.pkg` payload through the known package-backed manifest rules
- package header matches a known `PKGV00xx` variant currently supported by the sniff layer
- runtime setup does not require entry enumeration beyond what is already proven
- any package variant needing deeper internal parsing is rejected clearly as unsupported for now

This means “supported” in the next phase should mean “runtime can proceed using the package-backed payload model,” not “all scene internals are understood.”

## Architecture Implications

### 1. `lwe-library`

`lwe-library` should continue to own package-backed scene manifest synthesis.

Its responsibilities should be:

- preserve package provenance
- validate the known `scene.pkg` signature facts
- expose package-backed runtime payload information
- refuse unsupported or contradictory layouts clearly

### 2. `lwe-engine`

`lwe-engine` should stop expecting a guessed unpacked entry file as the scene identity.

The next runtime iteration should treat `scene.pkg` as the authoritative scene payload path for:

- runtime target identity
- apply barrier identity
- status reporting

If the engine later gains package-internal parsing, that should be an internal detail of the scene runtime, not part of the external runtime target contract.

### 3. App shell / desktop apply

The Tauri/app-shell layer should keep using typed runtime targets, but the scene branch should now be clearly package-backed. It should not need to know whether a later scene runtime internally resolves package entries.

## Failure Semantics

The next runtime phase must distinguish at least these cases:

- declared logical scene entry exists only as metadata, but a supported `scene.pkg` payload is present
- `scene.pkg` is missing
- `scene.pkg` exists but does not match a supported `PKGV00xx` header
- `scene.pkg` header is known, but the runtime still cannot proceed with the current package-backed subset

These failure classes are more honest and more actionable than collapsing everything into “missing scene.json”.

## Testing Strategy

### 1. Manifest and package-backed runtime target tests

Continue verifying that package-backed scene projects resolve through `scene.pkg` and preserve provenance.

### 2. Engine contract tests

The next engine tests should verify:

- scene runtime identity is the package path, not a guessed logical entry path
- apply success/failure is tied to the package-backed scene target
- unsupported package variants fail clearly before false success is reported

### 3. Real-sample validation

Real local scene samples should remain the ground truth for this phase.

At least one local subscribed scene sample should be used to verify that:

- its package-backed manifest resolves correctly
- it is either accepted by the package-backed runtime subset or rejected for a clearly stated reason

If the runtime cannot proceed past package-backed acceptance for any real sample, that should be treated as a real product limitation, not hidden behind optimistic compatibility labels.

## Risks and Trade-Offs

### 1. Package-backed runtime may still be too weak

Treating `scene.pkg` as the payload is more honest, but it does not magically solve internal scene parsing. The next runtime phase may still be blocked if first-frame rendering requires package internals we cannot yet access.

### 2. Future internal parsing may change details

If a later reader/indexer reveals a stable internal entry model, some runtime internals may evolve. The design keeps this safe by making `scene.pkg` the external contract now and leaving internal package understanding as an implementation detail later.

### 3. Variant fragmentation

Observed versions are `PKGV0021` and `PKGV0023`, but more variants may exist. The next implementation phase should explicitly scope which variants are accepted, not silently treat all `PKGV` headers as equivalent.

## Success Criteria

This design is successfully implemented when:

- the scene runtime no longer depends on top-level `scene.json` as an input artifact
- `scene.pkg` is the authoritative external runtime payload for supported scene projects
- scene runtime identity, apply barrier identity, and status identity all align on the package-backed model
- unsupported package variants fail clearly instead of being treated as if they were parsed scene entries
- the next runtime phase can proceed without relying on invented package enumeration semantics
