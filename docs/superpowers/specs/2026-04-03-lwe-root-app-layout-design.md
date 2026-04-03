# LWE Root App Layout Design

## Summary

Promote the current single active application from `apps/lwe` to the repository root so the root directory itself becomes the sole LWE app workspace. This removes the now-redundant `apps/lwe` wrapper layer while preserving the retained Rust crates under `crates/`.

This change is motivated by the current repository shape: there is only one active frontend/Tauri app, no root-level Node workspace, and many commands already conceptually treat the repository as a single-product codebase. Keeping the app buried under `apps/lwe` now adds path churn without providing real monorepo value.

## Goals

- Make the repository root the single app root for LWE.
- Remove the `apps/lwe` wrapper layer entirely.
- Keep `crates/lwe-core`, `crates/lwe-library`, and `crates/lwe-engine` where they are.
- Preserve Tauri, Svelte, Vite, pnpm, and Rust workflows with simpler root-level commands.
- Update scripts, docs, and workspace configuration so the new structure is coherent end-to-end.

## Non-Goals

- Do not redesign the retained Rust crates.
- Do not convert the repository into a multi-app root workspace.
- Do not keep long-term dual-path compatibility for both root and `apps/lwe`.
- Do not refactor unrelated application logic while moving files.
- Do not change product behavior beyond path/layout updates required by the migration.

## Current State

The repository currently has:

- no root-level `package.json`
- no root-level `pnpm-workspace.yaml`
- one active frontend app at `apps/lwe`
- one active Tauri crate at `apps/lwe/src-tauri`
- a Rust workspace whose only app member is `apps/lwe/src-tauri`

This means `apps/lwe` is acting as a historical nesting layer for a single application rather than as one member of a meaningful multi-app monorepo.

## Target Layout

After migration, the root directory should contain the active app directly:

- `src/`
- `static/` if present
- `src-tauri/`
- `package.json`
- `pnpm-lock.yaml`
- `vite.config.ts`
- `svelte.config.js`
- `tsconfig.json`
- `tailwind.config.ts`
- `postcss.config.cjs`
- `components.json`

And retain:

- `crates/`
- `docs/`
- `scripts/`
- root `Cargo.toml`

The `apps/` directory should no longer be required after the migration is complete.

## Architecture Impact

### 1. Node/Tauri app root

The repository root becomes the only app root. All Node-related commands should work directly from the repository root:

- `pnpm install`
- `pnpm test`
- `pnpm check`
- `pnpm exec vite`
- `cargo tauri dev` from the root app context

This removes the current need for `pnpm --dir apps/lwe ...` and `cd apps/lwe` patterns.

### 2. Rust workspace

The root Rust workspace should update the active app member from:

- `apps/lwe/src-tauri`

to:

- `src-tauri`

`src-tauri/Cargo.toml` path dependencies to the retained crates must be shortened accordingly.

### 3. Tauri configuration

The Tauri crate remains a subdirectory named `src-tauri/`, but it now lives directly under the root app. Any path-sensitive Tauri config or dev workflow assumptions must be updated to the new root-relative layout.

## Migration Strategy

This should be a one-step structural migration rather than a prolonged dual-path transition.

Recommended sequence:

1. move app files from `apps/lwe/` to the root
2. update root `Cargo.toml` workspace member paths
3. update `src-tauri/Cargo.toml` crate dependency paths
4. update root scripts/config/docs that reference `apps/lwe`
5. verify frontend and backend commands from the root
6. remove the now-empty `apps/` wrapper if nothing else depends on it

The migration should leave only one canonical location for the active app.

## Configuration Surfaces That Must Be Updated

This migration is mostly about path correctness. The high-risk surfaces are:

- root `Cargo.toml`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `Makefile`
- any VS Code settings/tasks that hardcode `apps/lwe`
- docs/specs/plans referencing app paths
- shell scripts under `scripts/`
- any CI or GitHub workflow files that call `pnpm --dir apps/lwe` or similar

The migration is only complete when these references are either updated or intentionally documented as historical references.

## Command Model After Migration

The desired post-migration command model is:

- install: `pnpm install`
- frontend tests: `pnpm test`
- frontend checks: `pnpm check`
- backend tests: `cargo test -p lwe-shell`
- dev: root-level Tauri/Vite workflow without `apps/lwe` path prefixes

This simpler command model is one of the primary reasons for the migration.

## Documentation Strategy

Documentation must be treated as part of the migration, not as a later cleanup task.

That includes:

- active product docs
- active superpowers specs/plans that still describe live paths
- README usage examples
- Makefile comments and command examples

Historical archived docs may keep old paths if they are clearly archival, but active docs should not keep teaching `apps/lwe` once the migration is done.

## Verification Strategy

The migration should be verified from the new root layout only.

Minimum required verification:

- `pnpm install`
- `pnpm test`
- `pnpm check`
- `cargo test -p lwe-shell`
- a Tauri dev launch using the new root layout

If any of these still requires `apps/lwe`-relative commands, the migration is incomplete.

## Risks and Trade-Offs

### 1. Wide path churn

This is a layout migration, so many files may change even though little product behavior changes. The implementation must stay disciplined and avoid unrelated refactors.

### 2. Hidden path references

The main failure mode is missing a path reference in docs, scripts, or config. This is why grep-driven verification is important.

### 3. Docs drift

If active docs are not updated, the repository will become confusing immediately after migration. Documentation updates are required, not optional.

## Success Criteria

This migration is complete when:

- the active LWE app lives at the repository root
- `apps/lwe` is no longer the canonical app path
- root-level frontend commands work without `--dir apps/lwe`
- the Rust workspace references `src-tauri` directly
- Tauri dev/test/check workflows work from the new root layout
- active scripts and docs no longer depend on `apps/lwe`
