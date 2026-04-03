# Retained Core Crate Rename Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename the retained core crates from `wayvid-*` to `lwe-*` and update workspace paths, Rust imports, package manifests, doctests, and active documentation so the active LWE codebase uses one consistent crate vocabulary.

**Architecture:** This plan renames only the retained active capability crates: `wayvid-core`, `wayvid-library`, and `wayvid-engine`. The legacy GUI/CLI crates have already been retired from the active workspace, so the rename can focus on the new LWE shell and the retained capability stack without preserving old product paths as active peers.

**Tech Stack:** Rust workspace, Cargo, Tauri, Markdown docs, doctests, Git

---

## Scope Note

This plan covers only the retained active core crates:

- `crates/wayvid-core` -> `crates/lwe-core`
- `crates/wayvid-library` -> `crates/lwe-library`
- `crates/wayvid-engine` -> `crates/lwe-engine`

It does **not**:

- revive or rename the retired legacy crates `wayvid-gui` / `wayvid-ctl`
- redesign the internals of those retained crates beyond the rename
- rename archived OpenSpec history that truthfully references the old crate names

## File Map

### Files to create

- `crates/lwe-core/**` - renamed shared-models crate path
- `crates/lwe-library/**` - renamed library/workshop crate path
- `crates/lwe-engine/**` - renamed runtime/rendering crate path

### Files to modify

- `Cargo.toml` - workspace members and comments for the renamed retained crates
- `src-tauri/Cargo.toml` - dependency path and package rename updates
- all Rust sources under `src-tauri/**` that import `wayvid_library`, `wayvid_core`, or `wayvid_engine`
- Rust sources under retained crates that import one another
- active docs and plans that reference the retained crates by name or path, including:
  - `README.md`
  - `docs/product/repository-reset.md`
  - `docs/product/roadmap.md`
  - `docs/superpowers/plans/2026-03-27-workshop-browsing-and-acquisition.md`
  - `docs/superpowers/plans/2026-03-30-rust-core-layering.md`
  - `docs/superpowers/plans/2026-03-30-legacy-gui-ctl-retirement.md`
  - `scripts/README.md`

### Files to inspect while implementing

- `crates/wayvid-core/Cargo.toml`
- `crates/wayvid-library/Cargo.toml`
- `crates/wayvid-engine/Cargo.toml`
- `src-tauri/Cargo.toml`
- `Cargo.toml`
- current import paths under `src-tauri/src/**`
- doctests in retained crates, especially:
  - `crates/wayvid-core/src/config/pattern.rs`
  - `crates/wayvid-library/src/lib.rs`

## Task 1: Rename `wayvid-core` to `lwe-core`

**Files:**
- Create by rename: `crates/lwe-core/**`
- Modify: `Cargo.toml`
- Modify: `crates/lwe-core/Cargo.toml`
- Modify: all active imports/docs that reference `wayvid-core` / `wayvid_core`
- Test: `cargo test -p lwe-core`

- [ ] **Step 1: Write the failing crate-name/import check**

Run this before changing files:

```bash
python3 - <<'PY'
from pathlib import Path
root = Path('Cargo.toml').read_text()
assert 'crates/lwe-core' in root
assert 'wayvid-core' not in root
print('core rename wired')
PY
```

Expected: FAIL because the workspace still references `crates/wayvid-core`.

- [ ] **Step 2: Rename the directory and package manifest**

Rename the directory:

```text
crates/wayvid-core -> crates/lwe-core
```

Update `crates/lwe-core/Cargo.toml`:

```toml
[package]
name = "lwe-core"
description = "Core types and configuration for LWE"
```

Keep version, edition, license, and repository fields unchanged for now.

- [ ] **Step 3: Update workspace membership and import paths**

In root `Cargo.toml`, replace:

```toml
"crates/wayvid-core",
```

with:

```toml
"crates/lwe-core",
```

Update references from `wayvid-core` / `wayvid_core` to `lwe-core` / `lwe_core` across active paths, including:

- `src-tauri/**`
- `crates/lwe-library/**`
- `crates/lwe-engine/**`
- `scripts/README.md`
- active plans/docs listed above

Do **not** rewrite archived OpenSpec history in this task.

- [ ] **Step 4: Update doctests and compile-sensitive docs**

Fix any compile-sensitive docs to use the new path, for example in `crates/lwe-core/src/config/pattern.rs`:

```rust
/// use lwe_core::config::matches_pattern;
```

- [ ] **Step 5: Run tests and commit**

Run:

```bash
cargo test -p lwe-core
```

Expected: PASS

Then:

```bash
git add Cargo.toml crates/lwe-core src-tauri docs scripts
git commit -m "refactor: rename wayvid-core to lwe-core"
```

## Task 2: Rename `wayvid-library` to `lwe-library`

**Files:**
- Create by rename: `crates/lwe-library/**`
- Modify: `Cargo.toml`
- Modify: `crates/lwe-library/Cargo.toml`
- Modify: `src-tauri/Cargo.toml`
- Modify: all active imports/docs that reference `wayvid-library` / `wayvid_library`
- Test: `cargo test -p lwe-library`

- [ ] **Step 1: Write the failing library-rename check**

Run before changes:

```bash
python3 - <<'PY'
from pathlib import Path
root = Path('Cargo.toml').read_text()
shell = Path('src-tauri/Cargo.toml').read_text()
assert 'crates/lwe-library' in root
assert 'lwe-library = { path = "../../../crates/lwe-library" }' in shell
print('library rename wired')
PY
```

Expected: FAIL because both files still reference `wayvid-library`.

- [ ] **Step 2: Rename the directory and package manifest**

Rename the directory:

```text
crates/wayvid-library -> crates/lwe-library
```

Update `crates/lwe-library/Cargo.toml`:

```toml
[package]
name = "lwe-library"
description = "Wallpaper library management for LWE (SQLite + metadata)"

[dependencies]
lwe-core = { path = "../lwe-core" }
```

- [ ] **Step 3: Update workspace membership, path dependencies, and imports**

Replace active references from `wayvid-library` / `wayvid_library` to `lwe-library` / `lwe_library` across:

- `Cargo.toml`
- `src-tauri/Cargo.toml`
- `src-tauri/src/**`
- `crates/lwe-engine/**` if it imports the library crate in any active path
- active docs/plans/scripts

Keep archive references untouched.

- [ ] **Step 4: Update doctests and crate-level docs**

In `crates/lwe-library/src/lib.rs`, update crate docs such as:

```rust
//! lwe-library: Wallpaper library management for LWE
//! use lwe_library::{LibraryDatabase, FolderScanner, ThumbnailGenerator};
```

Also update any `pub use wayvid_core::...` re-exports to `pub use lwe_core::...`.

- [ ] **Step 5: Run tests and commit**

Run:

```bash
cargo test -p lwe-library
```

Expected: PASS

Then:

```bash
git add Cargo.toml src-tauri/Cargo.toml crates/lwe-library src-tauri docs scripts
git commit -m "refactor: rename wayvid-library to lwe-library"
```

## Task 3: Rename `wayvid-engine` to `lwe-engine`

**Files:**
- Create by rename: `crates/lwe-engine/**`
- Modify: `Cargo.toml`
- Modify: `crates/lwe-engine/Cargo.toml`
- Modify: active imports/docs that reference `wayvid-engine` / `wayvid_engine`
- Test: `cargo test -p lwe-engine`

- [ ] **Step 1: Write the failing engine-rename check**

Run before changes:

```bash
python3 - <<'PY'
from pathlib import Path
root = Path('Cargo.toml').read_text()
assert 'crates/lwe-engine' in root
assert 'wayvid-engine' not in root
print('engine rename wired')
PY
```

Expected: FAIL because the workspace still references `wayvid-engine`.

- [ ] **Step 2: Rename the directory and package manifest**

Rename the directory:

```text
crates/wayvid-engine -> crates/lwe-engine
```

Update `crates/lwe-engine/Cargo.toml`:

```toml
[package]
name = "lwe-engine"
description = "Video rendering engine for LWE (Wayland + MPV)"

[dependencies]
lwe-core = { path = "../lwe-core" }
```

- [ ] **Step 3: Update workspace membership, path dependencies, and imports**

Replace active references from `wayvid-engine` / `wayvid_engine` to `lwe-engine` / `lwe_engine` across:

- `Cargo.toml`
- `src-tauri/**`
- `crates/lwe-engine/**`
- active docs/plans/scripts

Leave archive references alone.

- [ ] **Step 4: Update compile-sensitive docs and re-exports**

Adjust any active re-exports or compile-sensitive examples inside `crates/lwe-engine/**` to use `lwe_core` instead of `wayvid_core` and `lwe_engine` instead of `wayvid_engine` where relevant.

- [ ] **Step 5: Run tests and commit**

Run:

```bash
cargo test -p lwe-engine
```

Expected: PASS

Then:

```bash
git add Cargo.toml crates/lwe-engine src-tauri docs scripts
git commit -m "refactor: rename wayvid-engine to lwe-engine"
```

## Task 4: Sweep the Active Workspace and Docs for Old Retained-Core Names

**Files:**
- Modify: active docs/scripts if any old retained-core names remain
- Test: `cargo metadata --no-deps && cargo test -p lwe-app-shell`

- [ ] **Step 1: Verify the active workspace metadata uses only `lwe-*` retained core crates**

Run:

```bash
cargo metadata --no-deps > /tmp/lwe-retained-core.json && python3 - <<'PY'
import json
from pathlib import Path
data = json.loads(Path('/tmp/lwe-retained-core.json').read_text())
members = '\n'.join(data['workspace_members'])
assert 'lwe-core' in members
assert 'lwe-library' in members
assert 'lwe-engine' in members
assert 'wayvid-core' not in members
assert 'wayvid-library' not in members
assert 'wayvid-engine' not in members
print('retained core workspace renamed')
PY
```

Expected: prints `retained core workspace renamed`.

- [ ] **Step 2: Verify active source/docs/scripts no longer use the old retained-core imports**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
import subprocess

patterns = [
    'wayvid_core',
    'wayvid_library',
    'wayvid_engine',
    'wayvid-core',
    'wayvid-library',
    'wayvid-engine',
]

paths = [
    'src-tauri',
    'crates/lwe-core',
    'crates/lwe-library',
    'crates/lwe-engine',
    'README.md',
    'docs/product',
    'docs/superpowers/plans/2026-03-27-workshop-browsing-and-acquisition.md',
    'docs/superpowers/plans/2026-03-30-rust-core-layering.md',
    'docs/superpowers/plans/2026-03-30-legacy-gui-ctl-retirement.md',
    'scripts/README.md',
]

for pattern in patterns:
    result = subprocess.run(
        ['rg', '-n', pattern, *paths],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 1, result.stdout

print('active references renamed')
PY
```

Expected: prints `active references renamed`.

- [ ] **Step 3: Run final active-path tests**

Run:

```bash
cargo test -p lwe-app-shell
```

Expected: PASS

- [ ] **Step 4: Commit any final cleanup if needed**

If you had to make final small cleanup edits after the sweep, commit them here. Otherwise skip the commit.

If needed:

```bash
git add Cargo.toml  crates/lwe-core crates/lwe-library crates/lwe-engine README.md docs scripts
git commit -m "docs: complete retained core lwe rename sweep"
```

## Self-Review Checklist

- Spec coverage:
  - `wayvid-core -> lwe-core` → Task 1
  - `wayvid-library -> lwe-library` → Task 2
  - `wayvid-engine -> lwe-engine` → Task 3
  - workspace/path/import/doc sweep → Task 4
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Type/path consistency:
  - directory names, Cargo package names, Rust crate paths, and active docs all move together
  - retired legacy crates are not pulled back into active rename work

## Expected Output of This Plan

When this plan is complete, the active LWE repository path will use one retained-core vocabulary consistently:

- `lwe-core`
- `lwe-library`
- `lwe-engine`
- `lwe-app-shell`

This reduces the remaining `wayvid` identity to retired legacy crates and archived historical material only.

## Follow-on Plans After This One

The next plans after this file should cover:

1. deciding whether to archive or delete the retired `wayvid-gui` / `wayvid-ctl` directories entirely
2. any deeper workspace/package metadata cleanup after the retained-core rename settles
