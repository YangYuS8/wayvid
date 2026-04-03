# LWE Settings and Session Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unify LWE’s user-facing settings and session persistence so configuration lives under an `lwe` TOML-based config root, desktop assignment restore becomes part of the same persistence story, and the Settings UI reaches a more complete MVP state.

**Architecture:** This plan keeps Settings and desktop assignment persistence separate from library/database concerns, but aligns them under one coherent TOML-based configuration root. The implementation should migrate `wayvid`-named settings paths to `lwe`, fold desktop assignment state out of JSON and into TOML-backed persistence, route restore through that unified source, and finish the remaining Settings MVP UX polish (post-save collapse and Simplified Chinese).

**Tech Stack:** Rust workspace, Tauri, TOML/serde persistence, Svelte, pnpm

---

## Scope Note

This plan focuses on persistence coherence and the last Settings MVP gaps.

It includes:

- `wayvid -> lwe` config-root migration for active settings/session persistence
- moving desktop assignment/session state off JSON into TOML-backed persistence
- restoring applied wallpaper assignments from the unified persistence path
- polishing the Settings MVP save behavior
- adding Simplified Chinese as a user-selectable language

It does **not** include:

- a full configuration-center redesign
- advanced multi-profile settings
- unrelated runtime capability work

## File Map

### Files to create

- `src-tauri/src/results/session_persistence.rs` - result types for desktop assignment/session persistence under the TOML config root

### Files to modify

- `src-tauri/src/services/settings_persistence_service.rs` - move active settings path from `wayvid` to `lwe`
- `src-tauri/src/services/desktop_persistence_service.rs` - stop using standalone JSON as the active persistence format; route desktop assignment persistence through the TOML-based config story
- `src-tauri/src/services/desktop_service.rs` - restore assignments from the unified persistence source
- `src-tauri/src/services/library_service.rs` - consume the unified assignment source for quick-status projection
- `src-tauri/src/results/mod.rs` - export any new session-persistence result types if needed
- `src-tauri/src/models.rs` - extend models only as needed for the new settings/session persistence truth
- `src/lib/types.ts` - mirror any necessary frontend model changes
- `src/routes/settings/+page.svelte` - after successful save, collapse back to the settled view state and add Simplified Chinese option
- `src/routes/settings/page-render.test.ts` - update for the new save/collapse behavior and Chinese option
- `docs/product/roadmap.md` - reflect the unified settings/session persistence story once complete

### Files to inspect while implementing

- `src-tauri/src/services/settings_persistence_service.rs`
- `src-tauri/src/services/desktop_persistence_service.rs`
- `src-tauri/src/services/settings_service.rs`
- `src-tauri/src/services/desktop_service.rs`
- `src/routes/settings/+page.svelte`
- `docs/superpowers/specs/2026-03-31-lwe-settings-and-session-persistence-design.md`

## Task 1: Move the Active Config Root From `wayvid` to `lwe`

**Files:**
- Modify: `src-tauri/src/services/settings_persistence_service.rs`
- Test: `cargo test -p lwe-app-shell settings_persistence -- --nocapture`

- [x] **Step 1: Add a failing path test**

Add a new test in `settings_persistence_service.rs` asserting that the active user settings path resolves under `lwe`, not `wayvid`, for example:

```rust
#[test]
fn settings_path_uses_lwe_config_root() {
    let path = settings_path_from_env(
        Some(std::path::PathBuf::from("/tmp/config")),
        Some(std::path::PathBuf::from("/tmp/home")),
    )
    .unwrap();

    assert_eq!(path, std::path::PathBuf::from("/tmp/config/lwe/settings.toml"));
}
```

- [x] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p lwe-app-shell settings_persistence -- --nocapture
```

Expected: FAIL because the service still targets `wayvid/settings.toml`.

- [x] **Step 3: Change the active settings path to `lwe`**

Update `settings_persistence_service.rs` so the active path becomes:

```rust
path.join("lwe").join("settings.toml")
```

Keep the rest of the TOML logic unchanged for now.

- [x] **Step 4: Run verification and commit**

Run:

```bash
cargo test -p lwe-app-shell settings_persistence -- --nocapture
```

Expected: PASS

Then:

```bash
git add src-tauri/src/services/settings_persistence_service.rs
git commit -m "refactor: move active settings path to lwe config root"
```

## Task 2: Replace Standalone JSON Desktop Assignment Persistence With TOML-Based Session Persistence

**Files:**
- Create: `src-tauri/src/results/session_persistence.rs`
- Modify: `src-tauri/src/services/desktop_persistence_service.rs`
- Modify: `src-tauri/src/results/mod.rs`
- Test: `cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture`

- [x] **Step 1: Add a failing session-persistence round-trip test**

Add a test proving the desktop assignment state can round-trip through TOML, for example:

```rust
#[test]
fn desktop_assignment_persistence_round_trips_through_toml() {
    let path = temp_file_path();
    let service = DesktopPersistenceService::for_test(path.clone());

    assert!(service.save_assignment("eDP-1", "item-1").is_ok());

    let loaded = service.load_state();
    // assert assignment exists through the TOML-backed state
}
```

- [x] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture
```

Expected: FAIL because the current service still uses a separate JSON-backed shape.

- [x] **Step 3: Replace the active JSON persistence with TOML-backed assignment persistence**

Refactor `desktop_persistence_service.rs` so its active persistence format is TOML-based and lives under the same `lwe` config root. The persisted assignment facts remain minimal:

- `monitor_id -> item_id`

Keep the service-specific API shape (`load_state`, `save_assignment`, `clear_assignment`) if that still fits, but the on-disk representation should now be TOML-based and part of the unified config story.

If a small helper/result type makes this cleaner, add `results/session_persistence.rs` and export it from `results/mod.rs`.

- [x] **Step 4: Run verification and commit**

Run:

```bash
cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture
```

Expected: PASS

Then:

```bash
git add src-tauri/src/services/desktop_persistence_service.rs src-tauri/src/results/session_persistence.rs src-tauri/src/results/mod.rs
git commit -m "refactor: unify desktop assignment persistence under toml"
```

## Task 3: Route Restore Through the Unified TOML Persistence Source

**Files:**
- Modify: `src-tauri/src/services/desktop_service.rs`
- Modify: `src-tauri/src/services/library_service.rs`
- Test: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`

- [x] **Step 1: Add a failing restore-source test**

Add a test proving the desktop/library flow now reads assignment restore state from the unified TOML-backed persistence source, not the old JSON path.

- [x] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: FAIL until the restore source is switched over.

- [x] **Step 3: Update Desktop and Library services**

Make `desktop_service.rs` and `library_service.rs` consume the unified settings/session persistence source for assignment restore/quick-status projection.

The restore policy stays the same:

- monitor exists + item exists -> restore
- monitor missing -> explicit degraded state
- item missing -> explicit degraded state

Only the persistence source changes.

- [x] **Step 4: Run verification and commit**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: PASS

Then:

```bash
git add src-tauri/src/services/desktop_service.rs src-tauri/src/services/library_service.rs
git commit -m "refactor: restore desktop state from unified persistence"
```

## Task 4: Finish the Settings MVP UI Behavior

**Files:**
- Modify: `src/routes/settings/+page.svelte`
- Modify: `src/routes/settings/page-render.test.ts`
- Test: `pnpm --dir  test && pnpm --dir  check`

- [x] **Step 1: Add a failing UI-behavior test**

Add or update a test that verifies:

- Simplified Chinese is present as a selectable language option
- after a successful save, the editing controls collapse back to the settled/normal view state instead of staying expanded in an “editing” posture

- [x] **Step 2: Run the frontend tests to verify failure**

Run:

```bash
pnpm --dir  test
```

Expected: FAIL until the page behavior is updated.

- [x] **Step 3: Update the Settings page behavior**

Modify `settings/+page.svelte` so:

- `zh-CN` (or equivalent Simplified Chinese option) is added to the language selector
- after a successful save, the UI no longer feels “stuck open” in editing mode

Keep this minimal and consistent with the current UI foundation; do not redesign the entire Settings page.

- [x] **Step 4: Run verification and commit**

Run:

```bash
pnpm --dir  test && pnpm --dir  check
```

Expected: PASS

Then:

```bash
git add src/routes/settings/+page.svelte src/routes/settings/page-render.test.ts
git commit -m "feat: finish settings mvp ui behavior"
```

## Task 5: Update the Roadmap to Match the Unified Persistence Story

**Files:**
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertion over the roadmap

- [x] **Step 1: Update roadmap wording**

Adjust the `lwe-settings-mvp` wording so it now reflects:

- editable settings
- TOML-backed persistence under the `lwe` config root
- launch-on-login support
- session/assignment persistence alignment

- [x] **Step 2: Verify roadmap wording and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'TOML-backed persistence under the lwe config root' in roadmap or 'launch on login' in roadmap
print('settings/session roadmap wording updated')
PY
```

Expected: prints `settings/session roadmap wording updated`.

Then:

```bash
git add docs/product/roadmap.md
git commit -m "docs: update settings and session persistence roadmap"
```

## Self-Review Checklist

- Spec coverage:
  - `wayvid -> lwe` config root → Task 1
  - TOML unification for session persistence → Task 2
  - desktop/library restore path uses unified persistence → Task 3
  - Settings page save/collapse + zh-CN → Task 4
  - roadmap update → Task 5
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Scope check:
  - no broad settings-center expansion
  - no unrelated runtime work
  - no reintroduction of JSON as the active user-facing persistence format

## Expected Output of This Plan

When this plan is complete, LWE’s user-facing persistence story should finally feel coherent: configuration and desktop session state will live under `lwe`, use TOML, restore from one story, and present a more finished Settings MVP.
