# LWE Settings MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current read-only Settings page into a real, editable MVP for language, theme, launch-on-login, and Steam integration state, persisted through a dedicated TOML settings file.

**Architecture:** Keep Settings separate from desktop assignment persistence and separate from SQLite-backed library state. The implementation should add a typed TOML-backed settings store, an update input model, a Linux graphical-session autostart adapter, and a thin frontend editing surface that reads a Rust-owned snapshot and sends explicit updates back to Rust. This is a Settings MVP, not a full configuration center.

**Tech Stack:** Rust workspace, Tauri, TOML/serde persistence, Linux desktop autostart via `.desktop` autostart entry, Svelte, pnpm

---

## Scope Note

This plan covers only the first useful Settings slice:

- editable language
- editable theme
- editable launch-on-login
- visible Steam integration state
- real persistence and reload

It does **not** include:

- advanced compatibility settings
- advanced desktop policy tuning
- debug/logging controls
- `systemd --user` service management

## File Map

### Files to create

- `apps/lwe/src-tauri/src/results/settings_persistence.rs` - result types for settings load/save and autostart status
- `apps/lwe/src-tauri/src/services/settings_persistence_service.rs` - TOML settings file load/save helpers
- `apps/lwe/src-tauri/src/services/autostart_service.rs` - Linux graphical-session autostart helpers
- `apps/lwe/src-tauri/src/assembly/settings_page.rs` - may be extended or split to assemble editable settings state and status
- `apps/lwe/src-tauri/src/commands/settings.rs` - extended command surface for updating settings
- `apps/lwe/src/lib/components/SettingsSection.svelte` - optional small wrapper for grouped settings blocks if needed

### Files to modify

- `apps/lwe/src-tauri/src/models.rs` - add editable settings snapshot/update input models and autostart status fields
- `apps/lwe/src-tauri/src/results/mod.rs` - export settings persistence result types
- `apps/lwe/src-tauri/src/services/mod.rs` - export settings persistence and autostart services
- `apps/lwe/src-tauri/src/services/settings_service.rs` - stop returning a hard-coded placeholder snapshot; source data from persistence and autostart services
- `apps/lwe/src-tauri/src/assembly/action_outcome.rs` - add settings action outcome assembly if needed
- `apps/lwe/src-tauri/src/assembly/settings_page.rs` - build the richer settings snapshot for the frontend
- `apps/lwe/src/lib/types.ts` - mirror the richer settings models and update-input contract
- `apps/lwe/src/lib/ipc.ts` - add typed settings update commands
- `apps/lwe/src/routes/settings/+page.svelte` - replace the static snapshot display with editable controls
- `apps/lwe/src/lib/ui/select/*.svelte` and/or current form primitives only if needed for settings inputs
- `docs/product/roadmap.md` - reflect that Settings is now a real editable MVP once complete

### Files to inspect while implementing

- `apps/lwe/src-tauri/src/services/settings_service.rs`
- `apps/lwe/src-tauri/src/models.rs`
- `apps/lwe/src/routes/settings/+page.svelte`
- `apps/lwe/src/lib/types.ts`
- `apps/lwe/src/lib/ipc.ts`
- `docs/superpowers/specs/2026-03-31-lwe-settings-mvp-design.md`

## Task 1: Introduce Typed Settings Snapshot and Update Models

**Files:**
- Modify: `apps/lwe/src-tauri/src/models.rs`
- Modify: `apps/lwe/src/lib/types.ts`
- Test: `cargo test -p lwe-app-shell settings_service_returns_placeholder_result -- --nocapture`

- [ ] **Step 1: Add a failing serialization test for editable settings**

Add a new test in `apps/lwe/src-tauri/src/models.rs` asserting that a settings update input and richer snapshot serialize cleanly, for example:

```rust
#[test]
fn settings_models_serialize_editable_mvp_fields() {
    let update = SettingsUpdateInput {
        language: Some("en".to_string()),
        theme: Some("system".to_string()),
        launch_on_login: Some(true),
    };

    let value = serde_json::to_value(&update).unwrap();
    assert_eq!(value["language"], "en");
    assert_eq!(value["launchOnLogin"], true);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p lwe-app-shell settings_models_serialize_editable_mvp_fields -- --nocapture
```

Expected: FAIL because the new models do not exist yet.

- [ ] **Step 3: Add editable settings models in Rust**

Extend `apps/lwe/src-tauri/src/models.rs` with:

- `SettingsUpdateInput`
- richer `SettingsPageSnapshot` fields for:
  - `language`
  - `theme`
  - `launch_on_login`
  - `launch_on_login_available`
  - `steam_required`
  - `steam_status_message`
  - `stale`

Keep the model narrow to the approved MVP.

- [ ] **Step 4: Mirror the models into TypeScript**

Update `apps/lwe/src/lib/types.ts` with matching interfaces for the new snapshot and update input.

- [ ] **Step 5: Run tests and commit**

Run:

```bash
cargo test -p lwe-app-shell settings_models_serialize_editable_mvp_fields -- --nocapture
```

Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/models.rs apps/lwe/src/lib/types.ts
git commit -m "feat: add editable settings mvp models"
```

## Task 2: Implement TOML Settings Persistence

**Files:**
- Create: `apps/lwe/src-tauri/src/results/settings_persistence.rs`
- Create: `apps/lwe/src-tauri/src/services/settings_persistence_service.rs`
- Modify: `apps/lwe/src-tauri/src/results/mod.rs`
- Modify: `apps/lwe/src-tauri/src/services/mod.rs`
- Test: `cargo test -p lwe-app-shell settings_persistence -- --nocapture`

- [ ] **Step 1: Add a failing settings persistence test**

Create a test that proves the settings persistence service can round-trip the MVP settings set through a TOML file using a temp path.

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p lwe-app-shell settings_persistence -- --nocapture
```

Expected: FAIL because the service and result types do not exist yet.

- [ ] **Step 3: Add persistence result types**

Create `apps/lwe/src-tauri/src/results/settings_persistence.rs` with structured load/save results (for example `Loaded`, `Saved`, `Unavailable { reason }`).

- [ ] **Step 4: Implement the TOML persistence service**

Create `apps/lwe/src-tauri/src/services/settings_persistence_service.rs` so it:

- stores a small TOML settings file
- returns defaults if the file does not exist
- supports an explicit `for_test(path)` constructor
- only returns unavailable/error on real parse or I/O failure

- [ ] **Step 5: Export modules, verify, and commit**

Update `results/mod.rs` and `services/mod.rs`, then run:

```bash
cargo test -p lwe-app-shell settings_persistence -- --nocapture
```

Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/results/settings_persistence.rs apps/lwe/src-tauri/src/services/settings_persistence_service.rs apps/lwe/src-tauri/src/results/mod.rs apps/lwe/src-tauri/src/services/mod.rs
git commit -m "feat: add toml settings persistence"
```

## Task 3: Implement Graphical-Session Autostart Support

**Files:**
- Create: `apps/lwe/src-tauri/src/services/autostart_service.rs`
- Modify: `apps/lwe/src-tauri/src/services/mod.rs`
- Test: `cargo test -p lwe-app-shell autostart_service -- --nocapture`

- [ ] **Step 1: Add a failing autostart test**

Create a test that verifies the service can describe or create the expected graphical-session autostart entry path for LWE, using a temp directory override in tests.

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p lwe-app-shell autostart_service -- --nocapture
```

Expected: FAIL because the autostart service does not exist yet.

- [ ] **Step 3: Implement autostart service helpers**

Create `apps/lwe/src-tauri/src/services/autostart_service.rs` so it:

- targets graphical-session autostart only
- uses a desktop-entry-based path under the user config/autostart location
- can report current enabled/disabled state
- can enable/disable by creating/removing the entry
- supports a test path override

- [ ] **Step 4: Export, verify, and commit**

Update `services/mod.rs`, then run:

```bash
cargo test -p lwe-app-shell autostart_service -- --nocapture
```

Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/services/autostart_service.rs apps/lwe/src-tauri/src/services/mod.rs
git commit -m "feat: add graphical session autostart service"
```

## Task 4: Route Settings Through Real Services and Commands

**Files:**
- Modify: `apps/lwe/src-tauri/src/services/settings_service.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/settings_page.rs`
- Modify: `apps/lwe/src-tauri/src/commands/settings.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/action_outcome.rs` if needed
- Test: `cargo test -p lwe-app-shell settings_service -- --nocapture`

- [ ] **Step 1: Add a failing real-settings-service test**

Add a test that proves `SettingsService::load_page()` no longer returns a hard-coded placeholder snapshot and instead reflects persistence/autostart state.

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test -p lwe-app-shell settings_service -- --nocapture
```

Expected: FAIL because the current service is still placeholder-only.

- [ ] **Step 3: Replace placeholder settings service behavior**

Update `SettingsService` so it:

- loads persisted settings
- resolves current autostart state
- emits a truthful Steam status message
- returns a richer settings snapshot

Extend `commands/settings.rs` with an update command that accepts `SettingsUpdateInput`, persists changes, updates autostart where needed, and returns an updated snapshot or `ActionOutcome`.

- [ ] **Step 4: Assemble the richer settings snapshot**

Update `assembly/settings_page.rs` so the frontend gets a complete MVP snapshot and no longer depends on placeholder-only values.

- [ ] **Step 5: Run verification and commit**

Run:

```bash
cargo test -p lwe-app-shell settings_service -- --nocapture
```

Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/services/settings_service.rs apps/lwe/src-tauri/src/assembly/settings_page.rs apps/lwe/src-tauri/src/commands/settings.rs apps/lwe/src-tauri/src/assembly/action_outcome.rs
git commit -m "feat: connect settings mvp backend flows"
```

## Task 5: Replace the Read-Only Settings UI With an Editable MVP

**Files:**
- Create if needed: `apps/lwe/src/lib/components/SettingsSection.svelte`
- Modify: `apps/lwe/src/lib/ipc.ts`
- Modify: `apps/lwe/src/routes/settings/+page.svelte`
- Modify: `apps/lwe/src/lib/types.ts` if the UI contract changed during backend work
- Test: `pnpm --dir apps/lwe test && pnpm --dir apps/lwe check`

- [ ] **Step 1: Add a failing settings-page interaction test**

Add or update a frontend test so it expects:

- editable controls for language/theme/launch-on-login
- visible Steam integration state text
- no longer just a static read-only snapshot table

- [ ] **Step 2: Run the frontend tests to verify failure**

Run:

```bash
pnpm --dir apps/lwe test
```

Expected: FAIL once the new interaction expectations are in place.

- [ ] **Step 3: Implement the editable settings page**

Update `apps/lwe/src/routes/settings/+page.svelte` so it:

- renders editable controls for language/theme/launch-on-login
- loads the Rust-owned snapshot
- sends updates through the new settings command path
- reflects saved state truthfully
- shows Steam integration status in a readable MVP form

Use the existing UI foundation primitives rather than inventing a new visual system.

- [ ] **Step 4: Run verification and commit**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

Then:

```bash
git add apps/lwe/src/lib/ipc.ts apps/lwe/src/routes/settings/+page.svelte apps/lwe/src/lib/types.ts apps/lwe/src/lib/components/SettingsSection.svelte
git commit -m "feat: add editable settings mvp page"
```

## Task 6: Update the Roadmap to Reflect Settings MVP

**Files:**
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertion over the roadmap

- [ ] **Step 1: Update roadmap wording**

Adjust the roadmap to reflect that Settings is now a real editable MVP with TOML persistence and graphical-session autostart support.

- [ ] **Step 2: Verify roadmap wording and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'editable settings' in roadmap or 'launch on login' in roadmap
print('settings roadmap wording updated')
PY
```

Expected: prints `settings roadmap wording updated`.

Then:

```bash
git add docs/product/roadmap.md
git commit -m "docs: update settings mvp roadmap track"
```

## Self-Review Checklist

- Spec coverage:
  - editable language/theme/autostart settings → Tasks 1, 4, 5
  - TOML persistence → Task 2
  - graphical-session autostart → Task 3
  - Steam integration state visibility → Tasks 4, 5
  - roadmap update → Task 6
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Scope check:
  - no `systemd --user` management
  - no advanced settings-center expansion
  - TOML stays the only persistence mechanism for this MVP

## Expected Output of This Plan

When this plan is complete, `Settings` will stop being a read-only snapshot page and become a real MVP settings surface that can actually change, persist, and reload the core user-facing preferences LWE needs right now.
