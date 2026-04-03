# LWE AppImage Bundler Design

## Summary

Add a stable Linux AppImage packaging path for LWE using the Tauri v2 official bundler. This phase focuses on producing a reliable AppImage artifact from the current root-level app layout, using the repository-root `logo.svg` as the source asset for bundle icons.

The goal is not to solve every runtime distribution concern at once. The goal is to make `cargo tauri build --bundles appimage` a supported, repeatable packaging path for the project.

## Goals

- Support AppImage as the first official Linux bundle target.
- Use the Tauri v2 official bundler rather than custom packaging scripts.
- Integrate the repository-root `logo.svg` into the Tauri icon pipeline.
- Keep the new root app layout as the canonical packaging root.
- Verify that an AppImage artifact is produced successfully.

## Non-Goals

- Do not implement custom `linuxdeploy` or `appimagetool` workflows.
- Do not prioritize other Linux package targets in this phase.
- Do not solve every possible AppImage runtime dependency issue.
- Do not mix this work with scene runtime/rendering work.
- Do not leave duplicate packaging paths competing with the official Tauri bundler.

## Current State

The repository now has:

- the active app at the repository root
- `src-tauri/` as the active Tauri crate
- a root `logo.svg` provided for branding
- no finalized Tauri bundle configuration for a stable AppImage output path yet

The project should now treat the root layout as the packaging root and configure bundling from there.

## Design Principles

### 1. Use Tauri’s official bundle path

All packaging logic in this phase should flow through Tauri’s standard `bundle` configuration and `cargo tauri build --bundles appimage`.

### 2. Generate proper icon assets from the source SVG

The root `logo.svg` should be treated as the source-of-truth branding asset. The bundler should consume proper generated icon assets rather than depending on ad hoc runtime SVG handling.

### 3. Keep scope narrow

A successful AppImage build is the completion criterion. Runtime polish and wider Linux packaging coverage come later.

## Packaging Architecture

The packaging path should have three parts:

1. source branding asset
   - `logo.svg`

2. generated Tauri icon set
   - placed under the expected Tauri icon location for the root app layout

3. Tauri bundle configuration
   - `src-tauri/tauri.conf.json` bundle settings for Linux/AppImage

The generated icon set should become part of the application packaging inputs so the build is reproducible.

## Icon Strategy

The root `logo.svg` is the source asset, but AppImage packaging should not rely on the bundler magically interpreting the raw SVG alone.

The stable path is:

- keep `logo.svg` in the repository root as the editable master asset
- generate the icon assets Tauri expects for Linux packaging
- reference the generated icons from `src-tauri/tauri.conf.json`

This avoids subtle differences across environments and keeps the packaging inputs explicit.

## Bundle Configuration Direction

The Tauri config should be updated so that:

- Linux bundling explicitly includes AppImage in the supported targets for this phase
- bundle metadata uses the current root app layout
- icon references point to the generated icon assets

The configuration should not imply support for a broader Linux packaging matrix than this phase is actually validating.

## Build Command Contract

The project should support this packaging command as the canonical AppImage build path:

```bash
cargo tauri build --bundles appimage
```

This command should be enough to produce an AppImage artifact when run from the repository root in a properly provisioned Linux environment.

## Verification Strategy

This phase should verify:

1. icon pipeline correctness
   - generated icon assets exist where Tauri expects them

2. bundle configuration correctness
   - `tauri.conf.json` points at the correct bundle/icon metadata

3. artifact production
   - the AppImage build command completes successfully
   - an AppImage artifact appears in the Tauri bundle output directory

If the build fails, the result should be classified clearly as one of:

- configuration error
- icon or asset generation error
- missing system packaging dependency

## Documentation Expectations

The project should document the AppImage build command and any system-level prerequisites that are truly required for the official Tauri bundler path.

This documentation should describe the current supported scope honestly:

- AppImage supported first
- other Linux targets not yet prioritized in this phase

## Risks and Trade-Offs

### 1. System packaging dependencies

Tauri’s official Linux bundling path may still depend on host packages or tools. That is acceptable as long as failures are surfaced clearly and the configuration itself is sound.

### 2. Icon asset mismatches

If the generated icon set is incomplete or incorrectly referenced, the build may succeed with poor branding or fail during bundling. This is why icon generation must be treated as part of the design, not as an afterthought.

### 3. Scope creep into runtime distribution

This phase should stop at producing a stable AppImage artifact. It should not expand into solving every downstream runtime issue unless the build itself requires it.

## Success Criteria

This phase is complete when:

- `logo.svg` is integrated into a reproducible Tauri icon asset pipeline
- `src-tauri/tauri.conf.json` is configured for AppImage bundling under the new root layout
- `cargo tauri build --bundles appimage` succeeds in the current environment or fails only on clearly documented external packaging prerequisites
- an AppImage artifact is produced in the expected Tauri bundle output directory
