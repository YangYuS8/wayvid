# LWE Makefile Entrypoints Design

## Goal

Add a minimal top-level `Makefile` that shortens the most common LWE development commands without replacing the existing shell scripts or introducing a second build system.

## Scope

The `Makefile` should only provide these entrypoints:

- `install`
- `dev`
- `test`
- `check`

It should not add release, cleanup, packaging, or legacy-crate targets.

## Commands

### `install`

Installs frontend dependencies with pnpm:

```bash
pnpm --dir apps/lwe install
```

### `dev`

Runs the active LWE shell in development mode from `apps/lwe`:

```bash
cd apps/lwe && cargo tauri dev
```

### `test`

Runs the most common Rust and frontend test commands:

```bash
cargo test -p lwe-app-shell
pnpm --dir apps/lwe test
```

### `check`

Runs the common verification path for the active shell:

```bash
cargo test -p lwe-app-shell
pnpm --dir apps/lwe check
```

## Non-Goals

- Do not wrap every existing script in `scripts/`
- Do not add release/build/package targets
- Do not include retired legacy crate commands
- Do not change the underlying development workflow

## Rationale

The repository already has useful shell scripts for broader validation. This `Makefile` is intentionally a thin convenience layer for the most common local development loop only.
