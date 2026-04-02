# LWE Scene Package Investigation Design

## Summary

Investigate the real structure of Wallpaper Engine `scene.pkg` files so LWE can stop guessing scene entrypoints from `project.json.file` and instead build scene manifests from actual package contents. This design focuses on discovering the package format, entry structure, and minimal extractable metadata needed for later runtime work.

The immediate goal is not to implement full scene rendering. The goal is to replace a now-proven-wrong assumption with evidence-based package understanding.

## Problem Statement

Real subscribed scene projects on the current machine show a mismatch between metadata and on-disk files:

- `project.json` declares `file: "scene.json"`
- the project directory contains `scene.pkg`, not `scene.json`

This means the current model is likely wrong:

- `project.json.file` may be a logical or packaged entrypoint, not a real top-level filesystem file
- `scene.pkg` likely contains the real scene data and resource layout
- current `SceneManifest` construction from top-level disk files is therefore incomplete or wrong

Until this is understood, both the compatibility layer and the minimal native runtime risk being built on false assumptions.

## Goals

- Determine whether `scene.pkg` is a standard container, a custom package format, or a lightly wrapped archive.
- Determine whether `scene.pkg` contains the real `scene.json` entrypoint and how it is represented.
- Determine whether package contents can be enumerated and whether resource references can be surfaced without full extraction.
- Define the minimal parser output LWE needs from `scene.pkg` for the next scene-manifest/runtime iteration.
- Replace the assumption that a top-level file path from `project.json` is sufficient for scene runtime support.

## Non-Goals

- Do not implement a full generic package extraction tool unless investigation proves it is trivial and necessary.
- Do not implement a complete scene runtime in this investigation step.
- Do not promise support for all package variants discovered in the wild.
- Do not redesign unrelated app-shell or frontend code.

## Current Evidence

From real local subscribed samples already inspected:

- multiple scene projects under `~/.local/share/Steam/steamapps/workshop/content/431960/<id>/`
- `project.json` contains `"type": "scene"` and `"file": "scene.json"`
- the visible directory contents include:
  - `project.json`
  - `preview.*`
  - `scene.pkg`
- the named `scene.json` file is not present beside `project.json`

This strongly suggests the runtime entry is packaged inside `scene.pkg` rather than existing as a plain top-level file.

## Investigation Strategy

### 1. Use multiple real samples

Investigate at least 2 to 3 real subscribed scene packages from the current machine rather than relying on one sample. The purpose is to distinguish format-wide facts from project-specific accidents.

For each sample, gather:

- `project.json`
- top-level directory listing
- `file`/magic information for `scene.pkg`
- any extractable strings, headers, or index structures

### 2. Start with non-destructive structural inspection

Before attempting extraction, determine:

- file header signature
- whether the format matches zip/tar/sqlite/known archive patterns
- whether compressed or embedded filenames are visible through strings/index blocks

This keeps the first pass low-risk and prevents premature commitment to a parser architecture.

### 3. Escalate to a minimal reader only if structure is clear enough

If the package format is clear and stable enough after inspection, the next step can define a minimal reader that supports:

- reading package headers
- enumerating contained entry names
- locating the logical scene entry and immediate asset references

But this investigation step does not require a full extractor unless that falls out naturally from the evidence.

## Expected Outputs from Investigation

The investigation should produce answers to these concrete questions:

1. What is `scene.pkg` at the container level?
2. Does it contain `scene.json`, and if so under what path/name?
3. Can package entries be listed without full decompression?
4. What is the smallest reliable “real scene entry” model LWE can use?
5. What package-derived data should replace the current `SceneManifest.entry_file` assumption?

## Proposed Parser Boundary After Investigation

This investigation is expected to lead to a new parser boundary roughly like:

- `ScenePackageProbe` or equivalent
  - identifies package type/signature
  - exposes basic package metadata
- `ScenePackageIndex` or equivalent
  - lists package entries
  - identifies the runtime entry and required packaged assets
- revised `SceneManifest`
  - built from package-derived reality, not from guessed top-level filesystem files

The exact type names are flexible, but the boundary should clearly separate:

- package/container understanding
- scene manifest synthesis
- later runtime consumption

## Sample Selection Rules

Choose samples that improve confidence:

- one minimal/simple scene if available
- one richer scene with more properties/effects if available
- preferably samples from different workshop authors

If all samples look identical structurally, that is useful evidence. If they differ, the design must document which common denominator is safe to rely on.

## Error Handling Expectations

Investigation should classify failures clearly, for example:

- unknown package signature
- package appears valid but cannot be indexed
- expected logical entry missing from package
- package structure differs across real samples

These classifications matter because the next manifest/runtime layer should be able to distinguish “unsupported package format” from “broken package” from “package parsed but runtime subset unsupported.”

## Testing Strategy

This step should be validated with real sample evidence, not synthetic guesses alone.

### 1. Real-sample inspection notes

For each sample used, record:

- workshop id
- top-level files
- package signature observations
- whether entry enumeration succeeded
- whether the logical scene entry was found

### 2. Minimal automated probes

If a package probe/reader is implemented in the next step, tests should verify:

- package signature detection
- entry listing for a known sample or trimmed fixture
- failure behavior for malformed or unsupported input

### 3. Decision checkpoint

This investigation is complete only when there is enough evidence to choose one of these next moves explicitly:

- implement a minimal `scene.pkg` reader/indexer
- revise `SceneManifest` to use package-derived entries
- or, if the package format is too opaque, document the blocker and alternatives

## Risks and Trade-Offs

### 1. Hidden complexity inside `scene.pkg`

The package may be a custom binary format, which could make the next parser step materially harder than expected. That is still valuable information and should stop false progress on the runtime side.

### 2. Sample bias

One author’s package layout might not represent the common case. This is why multiple local samples are required.

### 3. Investigation creep

It will be tempting to keep going until a full extractor exists. This step should stop once the package format and real scene-entry model are sufficiently understood to write the next implementation plan.

## Success Criteria

This investigation is complete when:

- at least 2 to 3 real local `scene.pkg` samples have been inspected
- the package/container structure is characterized with concrete evidence
- the relationship between `project.json.file` and `scene.pkg` is explained clearly
- the likely real runtime entrypoint model is identified
- the next implementation step can be specified based on package-derived facts rather than guessed top-level files
