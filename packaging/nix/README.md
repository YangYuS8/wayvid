# Nix Packaging Notes

This directory is now a legacy reference for the retired `wayvid-gui` and `wayvid-ctl` packaging work.

## Current Status

- The active application path is `apps/lwe/src-tauri`.
- The legacy GUI and CLI crates are no longer current deliverables.
- These Nix notes are preserved only so old packaging assumptions remain documented while retirement work lands.

## What This Directory Still Represents

- historical flake packaging experiments for pre-LWE binaries
- packaging context that may help future migration work
- legacy reference material, not a supported install or release path

## What It Does Not Represent

- a current Nix install flow for the active LWE app shell
- an active promise that `wayvid-gui` or `wayvid-ctl` should be built, run, or shipped by default
- a release checklist for current deliverables

## If You Are Working On The Active Product

Use the active workspace verification path from the repository root instead:

```bash
cargo metadata --no-deps
cargo test -p lwe-app-shell
```

Any future Nix support for LWE should be proposed as new packaging work rather than inferred from this retired legacy material.
