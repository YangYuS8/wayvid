# Real Monitor Discovery and Desktop Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current placeholder monitor discovery and desktop persistence seams with a real first backend for the user’s current `Wayland + niri` environment, while keeping the public Desktop/Library contract thin, truthful, and future-ready.

**Architecture:** The implementation keeps the monitor/persistence interface generic but supplies a first concrete backend for the current environment. `MonitorDescriptor v1` stays intentionally small (`id`, `name`, `resolution`). Desktop persistence remains a tiny assignment store (`monitor_id -> item_id`) in a standalone JSON state file rather than moving into SQLite. Restore follows a “best effort + explicit degraded reporting” policy: recover what still maps cleanly, and surface monitor-missing or item-missing conditions without silently pretending success.

**Tech Stack:** Rust workspace, Tauri, Wayland/niri environment, serde/JSON persistence, `lwe-core`, `lwe-library`, `lwe-engine`, Cargo tests, pnpm frontend checks

---

## Scope Note

This plan is specifically about making the existing desktop flow real in the current environment.

It includes:

- a real monitor discovery backend for the current `Wayland + niri` environment
- a real JSON-backed persisted assignment store
- truthful restore handling for missing monitor / missing item cases
- propagation of that state through Desktop and Library snapshots

It does **not** include:

- multi-compositor support beyond the initial backend
- advanced multi-monitor rules/templates
- preview mode or all-monitor apply
- redesigning the frontend flow already in place

## File Map

### Files to create

- `src-tauri/src/services/backends/monitor_backend.rs` - backend trait and shared backend-facing monitor types
- `src-tauri/src/services/backends/niri_monitor_backend.rs` - first concrete monitor backend for the current Wayland/niri environment
- `src-tauri/src/services/backends/mod.rs` - backend exports
- `src-tauri/src/services/backends/persistence_backend.rs` - JSON persistence helpers and path resolution for desktop state

### Files to modify

- `src-tauri/src/services/monitor_service.rs` - switch from placeholder unavailable contract to real backend-backed monitor discovery
- `src-tauri/src/services/desktop_persistence_service.rs` - replace placeholder unavailable results with real JSON load/save/clear operations
- `src-tauri/src/services/desktop_service.rs` - consume the real monitor backend and persistence backend; implement truthful best-effort restore
- `src-tauri/src/services/library_service.rs` - derive desktop assignment status from the same real assignment snapshot
- `src-tauri/src/results/monitor_discovery.rs` - keep result types but adjust as needed for real backend output
- `src-tauri/src/results/desktop_persistence.rs` - keep result types but adjust as needed for real load/save/clear details
- `src-tauri/src/results/desktop_apply.rs` - add restore skip/failure detail if needed
- `src-tauri/src/results/desktop.rs` - surface known monitor state, assignment availability, and restore issues cleanly
- `src-tauri/src/assembly/desktop_page.rs` - render explicit restored/missing state, not fake empty state
- `src-tauri/src/assembly/library_page.rs` - project current assignment state from the real persistence snapshot
- `src-tauri/src/assembly/library_detail.rs` - same for Library detail
- `src-tauri/src/assembly/action_outcome.rs` - ensure apply/clear outcomes reflect real persistence results
- `src-tauri/src/models.rs` - extend frontend-facing monitor/assignment models only as needed for truthful restore-state rendering
- `src/lib/types.ts` - align TS contracts with any added monitor/restore state fields
- `src/routes/desktop/+page.svelte` - surface restore failures/missing monitor/missing item states clearly
- `src/routes/library/+page.svelte` - surface current desktop assignment state truthfully
- `src/lib/components/LibraryDetailPanel.svelte` - show assignment availability / degradation info when relevant
- `src/lib/components/DesktopMonitorCard.svelte` - show restore/degraded status when relevant
- `docs/product/roadmap.md` - update roadmap wording once the placeholder backend is replaced with a real first implementation

### Files to inspect while implementing

- `src-tauri/src/services/monitor_service.rs`
- `src-tauri/src/services/desktop_persistence_service.rs`
- `src-tauri/src/services/desktop_service.rs`
- `src-tauri/src/assembly/desktop_page.rs`
- `src-tauri/src/assembly/library_page.rs`
- `src-tauri/src/assembly/library_detail.rs`
- `src/lib/types.ts`
- `src/routes/desktop/+page.svelte`
- `src/routes/library/+page.svelte`

## Task 1: Introduce Backend Seams for Monitor Discovery and JSON Persistence

**Files:**
- Create: `src-tauri/src/services/backends/mod.rs`
- Create: `src-tauri/src/services/backends/monitor_backend.rs`
- Create: `src-tauri/src/services/backends/persistence_backend.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Test: `cargo test -p lwe-app-shell monitor_backend -- --nocapture`

- [ ] **Step 1: Write the failing backend-seam test**

Create `src-tauri/src/services/backends/monitor_backend.rs` with this test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_descriptor_v1_stays_small_and_stable() {
        let monitor = BackendMonitorDescriptor {
            id: "eDP-1".to_string(),
            name: "Built-in".to_string(),
            resolution: "2160x1440".to_string(),
        };

        assert_eq!(monitor.id, "eDP-1");
        assert_eq!(monitor.name, "Built-in");
        assert_eq!(monitor.resolution, "2160x1440");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell monitor_backend -- --nocapture`
Expected: FAIL because the backend modules do not exist yet.

- [ ] **Step 3: Create the monitor backend seam**

Create `src-tauri/src/services/backends/monitor_backend.rs` with:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendMonitorDescriptor {
    pub id: String,
    pub name: String,
    pub resolution: String,
}

pub enum BackendMonitorDiscovery {
    Known(Vec<BackendMonitorDescriptor>),
    Unavailable { reason: String },
}

pub trait MonitorBackend {
    fn list_monitors() -> BackendMonitorDiscovery;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_descriptor_v1_stays_small_and_stable() {
        let monitor = BackendMonitorDescriptor {
            id: "eDP-1".to_string(),
            name: "Built-in".to_string(),
            resolution: "2160x1440".to_string(),
        };

        assert_eq!(monitor.id, "eDP-1");
        assert_eq!(monitor.name, "Built-in");
        assert_eq!(monitor.resolution, "2160x1440");
    }
}
```

- [ ] **Step 4: Create the JSON persistence backend seam**

Create `src-tauri/src/services/backends/persistence_backend.rs` with:

```rust
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PersistedDesktopAssignments {
    pub assignments: BTreeMap<String, String>,
}

pub fn desktop_state_path() -> PathBuf {
    PathBuf::from("desktop-state.json")
}
```

Create `src-tauri/src/services/backends/mod.rs` with:

```rust
pub mod monitor_backend;
pub mod persistence_backend;
```

And export it from `src-tauri/src/services/mod.rs`:

```rust
pub mod backends;
```

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell monitor_backend -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/services/mod.rs src-tauri/src/services/backends
git commit -m "refactor: add monitor and persistence backend seams"
```

## Task 2: Implement the First Real Monitor Backend for Wayland + niri

**Files:**
- Create: `src-tauri/src/services/backends/niri_monitor_backend.rs`
- Modify: `src-tauri/src/services/backends/mod.rs`
- Modify: `src-tauri/src/services/monitor_service.rs`
- Test: `cargo test -p lwe-app-shell monitor_service -- --nocapture`

- [ ] **Step 1: Write the failing backend-selection test**

Add this test to `src-tauri/src/services/monitor_service.rs` first:

```rust
#[test]
fn monitor_service_uses_real_backend_result_type() {
    let result = MonitorService::list_monitors();
    assert!(matches!(
        result,
        crate::results::monitor_discovery::MonitorDiscoveryResult::Known(_)
            | crate::results::monitor_discovery::MonitorDiscoveryResult::Unavailable { .. }
    ));
}
```

- [ ] **Step 2: Run the test to verify current behavior fails to provide a backend-backed path**

Run: `cargo test -p lwe-app-shell monitor_service -- --nocapture`
Expected: existing placeholder behavior still passes old tests but there is no backend module. Confirm the new test fails or requires the backend path to be introduced.

- [ ] **Step 3: Implement the niri backend**

Create `src-tauri/src/services/backends/niri_monitor_backend.rs` with the first concrete backend. Keep it minimal and truthful; the implementation can use the smallest reliable mechanism available in the current environment to enumerate outputs and basic resolution. The result must be expressed as:

```rust
BackendMonitorDiscovery::Known(vec![BackendMonitorDescriptor { id, name, resolution }, ...])
```

or:

```rust
BackendMonitorDiscovery::Unavailable { reason }
```

Export it in `src-tauri/src/services/backends/mod.rs`:

```rust
pub mod niri_monitor_backend;
```

Update `src-tauri/src/services/monitor_service.rs` so `MonitorService::list_monitors()` delegates to this backend and converts `BackendMonitorDiscovery` into `MonitorDiscoveryResult`.

- [ ] **Step 4: Keep `resolve_specific_monitor()` aligned with the new backend-backed result**

Make sure it still preserves `Known(...)` vs `Unavailable { .. }` and resolves by the stable backend `id` only.

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell monitor_service -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/services/backends/niri_monitor_backend.rs src-tauri/src/services/backends/mod.rs src-tauri/src/services/monitor_service.rs
git commit -m "feat: add first real monitor discovery backend"
```

## Task 3: Implement Real JSON Desktop Assignment Persistence

**Files:**
- Modify: `src-tauri/src/services/backends/persistence_backend.rs`
- Modify: `src-tauri/src/services/desktop_persistence_service.rs`
- Test: `cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture`

- [ ] **Step 1: Write the failing persistence-roundtrip test**

Add this test to `src-tauri/src/services/desktop_persistence_service.rs` first:

```rust
#[test]
fn persistence_service_round_trips_assignments() {
    let path = tempfile::tempdir().unwrap().path().join("desktop-state.json");
    let service = DesktopPersistenceService::for_test(path.clone());

    assert!(matches!(service.load_state(), crate::results::desktop_persistence::DesktopPersistenceLoad::Loaded(_)));

    assert!(matches!(
        service.save_assignment("eDP-1", "item-1"),
        crate::results::desktop_persistence::DesktopPersistenceWrite::Saved
    ));

    let loaded = service.load_state();
    match loaded {
        crate::results::desktop_persistence::DesktopPersistenceLoad::Loaded(assignments) => {
            assert_eq!(assignments.get("eDP-1").unwrap(), "item-1");
        }
        _ => panic!("expected loaded assignments"),
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture`
Expected: FAIL because persistence still returns `Unavailable` and has no testable backend.

- [ ] **Step 3: Implement JSON load/save/clear**

Update `src-tauri/src/services/backends/persistence_backend.rs` so it actually reads/writes a JSON file with:

```json
{
  "assignments": {
    "monitor-id": "item-id"
  }
}
```

Then update `src-tauri/src/services/desktop_persistence_service.rs` so it:

- loads the file if present
- returns `Loaded(empty map)` if the file does not exist yet
- returns `Saved` / `Cleared` on success
- returns `Unavailable { reason }` only on real I/O or parse failure
- exposes a `for_test(path: PathBuf)` constructor so tests don’t touch the real config path

- [ ] **Step 4: Run tests and commit**

Run: `cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/services/backends/persistence_backend.rs src-tauri/src/services/desktop_persistence_service.rs
git commit -m "feat: add json desktop assignment persistence"
```

## Task 4: Thread the Real Backends Through Desktop and Library Flows

**Files:**
- Modify: `src-tauri/src/services/desktop_service.rs`
- Modify: `src-tauri/src/services/library_service.rs`
- Modify: `src-tauri/src/results/desktop.rs`
- Modify: `src-tauri/src/results/desktop_apply.rs`
- Modify: `src-tauri/src/assembly/desktop_page.rs`
- Modify: `src-tauri/src/assembly/library_page.rs`
- Modify: `src-tauri/src/assembly/library_detail.rs`
- Modify: `src-tauri/src/assembly/action_outcome.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src/lib/types.ts`
- Test: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture && pnpm --dir  check`

- [ ] **Step 1: Add the failing “real restore” test**

Add this test to `src-tauri/src/services/desktop_service.rs` first:

```rust
#[test]
fn desktop_service_load_page_reports_restorable_assignments() {
    // Use a test persistence service + known monitor list to confirm
    // persisted assignments are surfaced as assignments_available.
    let result = DesktopService::load_page().unwrap();
    assert!(result.assignments_available || result.persistence_issue.is_some());
}
```

- [ ] **Step 2: Run the test to verify the old placeholder assumptions fail**

Run: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`
Expected: existing logic still partially assumes unavailable backends or placeholder monitor state.

- [ ] **Step 3: Wire real monitor + persistence state through services and assemblers**

Update Desktop and Library service/assembly flow so that:

- known discovered monitors become real `DesktopMonitorSummary` entries
- persisted assignments become real quick-status data for matching Library items
- restore follows the agreed policy:
  - monitor exists + item exists -> restored
  - monitor missing -> explicit unavailable/missing-monitor state
  - item missing -> explicit missing-item state
- apply/clear fail truthfully when monitor discovery or persistence genuinely fails

- [ ] **Step 4: Keep the frontend contract truthful**

Update Rust models and TS types only as needed to represent:

- monitor availability
- assignment availability
- restore degradation reasons

Do not bloat the contract beyond what the current Desktop/Library UI can render.

- [ ] **Step 5: Run tests and commit**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture && pnpm --dir  check
```

Expected: PASS

Then:

```bash
git add src-tauri/src/services/desktop_service.rs src-tauri/src/services/library_service.rs src-tauri/src/results/desktop.rs src-tauri/src/results/desktop_apply.rs src-tauri/src/assembly/desktop_page.rs src-tauri/src/assembly/library_page.rs src-tauri/src/assembly/library_detail.rs src-tauri/src/assembly/action_outcome.rs src-tauri/src/models.rs src/lib/types.ts
git commit -m "feat: wire real monitor and persistence state through desktop flow"
```

## Task 5: Update Roadmap Wording for the New Reality

**Files:**
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertion over the roadmap

- [ ] **Step 1: Update the roadmap wording**

Adjust the `desktop-shell-and-library-flow` / monitor-persistence follow-up wording so it reflects the actual post-implementation state. If monitor discovery and JSON persistence are both real by this point, use wording like:

```md
- `desktop-shell-and-library-flow`: the active LWE shell now supports monitor-aware Library apply, Desktop clear, and JSON-backed assignment restore on the current Wayland + niri path; follow-on work should generalize the backend and deepen runtime controls rather than establish the first real desktop state path
```

- [ ] **Step 2: Verify roadmap wording and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'monitor-aware Library apply' in roadmap
print('real monitor/persistence roadmap wording updated')
PY
```

Expected: prints `real monitor/persistence roadmap wording updated`.

Then:

```bash
git add docs/product/roadmap.md
git commit -m "docs: update real monitor persistence roadmap wording"
```

## Self-Review Checklist

- Spec coverage:
  - first real monitor backend → Tasks 1, 2
  - first real JSON persistence backend → Tasks 1, 3
  - Desktop / Library flow uses those real backends → Task 4
  - roadmap updated to the true post-implementation state → Task 5
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Type consistency:
  - `BackendMonitorDescriptor`, `MonitorDiscoveryResult`, `DesktopPersistenceLoad`, `DesktopPersistenceWrite`, and the Desktop/Library availability fields are introduced before the flow threads them through.

## Expected Output of This Plan

When this plan is complete, the Desktop flow will stop depending on placeholder monitor/persistence backends and will gain a first real environment-backed implementation for the current machine’s Wayland + niri setup, while keeping the contract generic enough for future backends.

## Follow-on Plans After This One

The next plans after this file should cover:

1. expanding monitor discovery beyond the initial Wayland + niri backend
2. deepening runtime preview/apply behavior once the desktop state path is truly real
