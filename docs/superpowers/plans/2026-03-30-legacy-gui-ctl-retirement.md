# Legacy GUI and CLI Retirement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove `crates/wayvid-gui` and `crates/wayvid-ctl` from the active LWE workspace and documentation story without deleting their directories yet, so the repository’s active product path is limited to `src-tauri` plus retained core crates.

**Architecture:** This plan retires legacy identity before retained-core renaming. The active workspace will shrink to the LWE shell and the retained core capabilities, while legacy GUI/CLI crates remain in the repository only as explicitly retired reference material. The work focuses on workspace membership, default verification paths, and documentation truthfulness; it does not yet move or delete the legacy directories.

**Tech Stack:** Rust workspace, Cargo, Markdown docs, OpenSpec-aware repository docs, Git

---

## Scope Note

This plan is about **retiring legacy crates from the active workspace**, not renaming retained core crates yet.

It does **not**:

- rename `lwe-core`, `lwe-library`, or `lwe-engine`
- move `crates/wayvid-gui` or `crates/wayvid-ctl` into a `legacy/` directory yet
- delete the old GUI/CLI source trees from git history
- rewrite archived OpenSpec material that truthfully references past GUI/CLI work

## File Map

### Files to modify

- `Cargo.toml` - remove legacy GUI/CLI crates from active workspace members and update workspace comments
- `README.md` - clarify the active LWE path and explicitly retire the old GUI/CLI crates from the current product surface
- `docs/product/repository-reset.md` - move `wayvid-gui` and `wayvid-ctl` out of retained migration candidates into a retired legacy section
- `docs/product/roadmap.md` - ensure roadmap language points future work at `src-tauri` and retained core crates only
- `docs/superpowers/plans/2026-03-27-workshop-browsing-and-acquisition.md` - remove assumptions that `wayvid-gui` / `wayvid-ctl` remain active workspace members
- `docs/superpowers/plans/2026-03-30-rust-core-layering.md` - make sure the layering plan no longer assumes legacy GUI/CLI crates are still active workspace peers

### Files to create

- `docs/archive/legacy-crates.md` - short archive note explaining that `crates/wayvid-gui` and `crates/wayvid-ctl` are retained only as legacy reference, not active product components

### Files to inspect while implementing

- `crates/wayvid-gui/Cargo.toml`
- `crates/wayvid-ctl/Cargo.toml`
- `src-tauri/Cargo.toml`
- `Cargo.toml`
- `docs/product/repository-reset.md`
- `README.md`

## Task 1: Remove Legacy GUI/CLI Crates from the Active Workspace

**Files:**
- Modify: `Cargo.toml`
- Test: `cargo metadata --no-deps`

- [ ] **Step 1: Write the failing workspace-membership test**

Create a temporary assertion script first by adding this test command to your notes and running it before editing `Cargo.toml`:

```bash
python3 - <<'PY'
from pathlib import Path
text = Path('Cargo.toml').read_text()
assert '"crates/wayvid-gui"' not in text
assert '"crates/wayvid-ctl"' not in text
print('workspace cleaned')
PY
```

Expected: FAIL because both legacy crates are still active workspace members.

- [ ] **Step 2: Remove `wayvid-gui` and `wayvid-ctl` from workspace members**

Edit `Cargo.toml` so the `members` list becomes:

```toml
members = [
    "crates/lwe-core",
    "crates/lwe-engine",
    "crates/lwe-library",
    "src-tauri",
]
```

Update the opening workspace comment block to remove these two lines:

```toml
# - wayvid-gui: retained desktop shell and UX pattern candidate
# - wayvid-ctl: retained command surface and automation candidate
```

Replace them with:

```toml
# - wayvid-gui: retired legacy GUI shell kept temporarily for reference only
# - wayvid-ctl: retired legacy CLI surface kept temporarily for reference only
```

- [ ] **Step 3: Verify the workspace resolves cleanly without the legacy crates**

Run:

```bash
cargo metadata --no-deps > /tmp/lwe-workspace-metadata.json && python3 - <<'PY'
from pathlib import Path
text = Path('Cargo.toml').read_text()
assert '"crates/wayvid-gui"' not in text
assert '"crates/wayvid-ctl"' not in text
print('workspace cleaned')
PY
```

Expected:

- `cargo metadata --no-deps` exits successfully
- the Python script prints `workspace cleaned`

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: retire legacy gui and cli crates from workspace"
```

## Task 2: Make the Repository Story Truthful About Legacy Crates

**Files:**
- Modify: `README.md`
- Modify: `docs/product/repository-reset.md`
- Create: `docs/archive/legacy-crates.md`
- Test: `python3` assertions over docs

- [ ] **Step 1: Write the failing documentation truthfulness check**

Run this before editing docs:

```bash
python3 - <<'PY'
from pathlib import Path
repo = Path('docs/product/repository-reset.md').read_text()
assert 'retired legacy GUI shell' in repo
assert 'retired legacy CLI surface' in repo
print('legacy docs aligned')
PY
```

Expected: FAIL because the reset inventory still treats both crates as retained migration candidates.

- [ ] **Step 2: Update `README.md` to narrow the active path**

Add a short section after `What Remains Valuable`:

```md
## Active Product Path

The active LWE product path is now limited to:

- `src-tauri`
- retained core crates for shared models, library/workshop logic, and runtime behavior

The legacy crates `crates/wayvid-gui` and `crates/wayvid-ctl` are no longer active workspace components. They remain in the repository temporarily as retired reference material only.
```

- [ ] **Step 3: Rewrite the reset inventory to retire the legacy crates**

In `docs/product/repository-reset.md`, change the `Retained Migration Candidates` list so it keeps only:

```md
- `crates/lwe-core` as a migration candidate for shared models, configuration, and cross-cutting types.
- `crates/lwe-engine` as a migration candidate for Linux wallpaper playback, rendering, and runtime integration knowledge.
- `crates/lwe-library` as a migration candidate for library indexing, metadata handling, and local asset management.
```

Then add a new section:

```md
## Retired Legacy Crates

- `crates/wayvid-gui` is a retired legacy GUI shell superseded by the LWE Tauri + Svelte application shell.
- `crates/wayvid-ctl` is a retired legacy CLI/control surface that is no longer part of the active LWE workspace.

Both directories remain temporarily in the repository for reference only and should not be treated as active product components.
```

- [ ] **Step 4: Create `docs/archive/legacy-crates.md`**

Write this file content:

```md
# Legacy Crates

`crates/wayvid-gui` and `crates/wayvid-ctl` have been retired from the active LWE workspace.

They remain in the repository temporarily for historical reference and selective migration lookup only.

They are not part of the active product path, default workspace verification surface, or future naming/unification work unless a later change explicitly revives some portion of their functionality.
```

- [ ] **Step 5: Verify docs are aligned and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
readme = Path('README.md').read_text()
repo = Path('docs/product/repository-reset.md').read_text()
archive = Path('docs/archive/legacy-crates.md').read_text()
assert 'active LWE product path' in readme
assert 'Retired Legacy Crates' in repo
assert 'retired legacy GUI shell' in repo
assert 'retired legacy CLI surface' in repo
assert 'retired from the active LWE workspace' in archive
print('legacy docs aligned')
PY
```

Expected: prints `legacy docs aligned`.

Then:

```bash
git add README.md docs/product/repository-reset.md docs/archive/legacy-crates.md
git commit -m "docs: mark legacy gui and cli crates as retired"
```

## Task 3: Align Plans and Roadmap With the Retired-Crate Decision

**Files:**
- Modify: `docs/product/roadmap.md`
- Modify: `docs/superpowers/plans/2026-03-27-workshop-browsing-and-acquisition.md`
- Modify: `docs/superpowers/plans/2026-03-30-rust-core-layering.md`
- Test: `python3` assertions over plan wording

- [ ] **Step 1: Write the failing plan-alignment check**

Run this before editing:

```bash
python3 - <<'PY'
from pathlib import Path
workshop = Path('docs/superpowers/plans/2026-03-27-workshop-browsing-and-acquisition.md').read_text()
layering = Path('docs/superpowers/plans/2026-03-30-rust-core-layering.md').read_text()
assert 'crates/wayvid-gui' not in workshop
assert 'crates/wayvid-ctl' not in workshop
assert 'wayvid-gui' not in layering
assert 'wayvid-ctl' not in layering
print('plans aligned')
PY
```

Expected: FAIL because older plan text still refers to the legacy crates as active workspace participants or retained peers.

- [ ] **Step 2: Update `docs/product/roadmap.md`**

Adjust the roadmap text so the active roadmap is explicit about the current product path. Add or update language like:

```md
The active application shell for ongoing work is `src-tauri`. Legacy crates `crates/wayvid-gui` and `crates/wayvid-ctl` have been retired from the active workspace and are not future-facing roadmap targets.
```

- [ ] **Step 3: Remove active-workspace assumptions from the Workshop plan**

In `docs/superpowers/plans/2026-03-27-workshop-browsing-and-acquisition.md`:

- remove any `members = [...]` examples that still include `crates/wayvid-gui` or `crates/wayvid-ctl`
- remove or replace any instruction that treats `crates/wayvid-gui/locales/*.toml` as an active shell dependency
- replace it with wording like:

```md
Legacy GUI locale files may be inspected only as wording references; they are not part of the active LWE shell.
```

- [ ] **Step 4: Remove active-peer assumptions from the Rust layering plan**

In `docs/superpowers/plans/2026-03-30-rust-core-layering.md`, update the architecture wording so the hard active crate boundaries are described as:

```md
Keep the active crate boundaries as `lwe-shell`, `lwe-library`, `lwe-core`, and `lwe-engine` for now. The retired legacy crates `wayvid-gui` and `wayvid-ctl` are outside the active workspace path.
```

Do not imply that `wayvid-gui` or `wayvid-ctl` are still active workspace peers.

- [ ] **Step 5: Verify plan alignment and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
workshop = Path('docs/superpowers/plans/2026-03-27-workshop-browsing-and-acquisition.md').read_text()
layering = Path('docs/superpowers/plans/2026-03-30-rust-core-layering.md').read_text()
assert 'retired from the active workspace' in roadmap
assert 'Legacy GUI locale files may be inspected only as wording references' in workshop
assert 'outside the active workspace path' in layering
print('plans aligned')
PY
```

Expected: prints `plans aligned`.

Then:

```bash
git add docs/product/roadmap.md docs/superpowers/plans/2026-03-27-workshop-browsing-and-acquisition.md docs/superpowers/plans/2026-03-30-rust-core-layering.md
git commit -m "docs: align plans with legacy crate retirement"
```

## Task 4: Verify the Active Workspace Path Is Clean

**Files:**
- Modify: `Cargo.toml` if a small comment adjustment is still needed after verification
- Test: `cargo metadata --no-deps && cargo test -p lwe-shell`

- [ ] **Step 1: Verify the active workspace path builds without legacy crates**

Run:

```bash
cargo metadata --no-deps > /tmp/lwe-active-workspace.json && cargo test -p lwe-shell
```

Expected:

- `cargo metadata --no-deps` succeeds
- `cargo test -p lwe-shell` succeeds

- [ ] **Step 2: Verify that the active workspace no longer includes legacy GUI/CLI crates**

Run:

```bash
python3 - <<'PY'
import json
from pathlib import Path
data = json.loads(Path('/tmp/lwe-active-workspace.json').read_text())
members = '\n'.join(data['workspace_members'])
assert 'wayvid-gui' not in members
assert 'wayvid-ctl' not in members
assert 'lwe-shell' in members
print('active workspace clean')
PY
```

Expected: prints `active workspace clean`.

- [ ] **Step 3: Commit any final workspace-comment cleanup if needed**

If you needed a final tiny adjustment to `Cargo.toml` comments after verification, commit it here. Otherwise, make no code change and skip the commit.

If a commit is needed:

```bash
git add Cargo.toml
git commit -m "chore: finalize active workspace after legacy retirement"
```

## Self-Review Checklist

- Spec coverage:
  - legacy GUI/CLI crates removed from active workspace → Task 1
  - docs and reset inventory describe them as retired → Task 2
  - roadmap and active plans stop treating them as active workspace peers → Task 3
  - active LWE verification path works without them → Task 4
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Type/wording consistency:
  - `retired legacy GUI shell`
  - `retired legacy CLI surface`
  - `active LWE workspace` / `active product path`
  are used consistently across the plan.

## Expected Output of This Plan

When this plan is complete, the repository will:

- stop treating `crates/wayvid-gui` and `crates/wayvid-ctl` as active workspace members
- document those crates as retired legacy reference material
- narrow the active LWE path to `src-tauri` plus retained core crates
- reduce confusion before the follow-on retained-core rename work

## Follow-on Plans After This One

The next plans after this file should cover:

1. renaming retained core crates from `wayvid-*` to `lwe-*`
2. optionally moving retired legacy crates into a dedicated `legacy/` or `archive/` location, or deleting them entirely once no longer needed
