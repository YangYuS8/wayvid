# Real Monitor Discovery and Desktop Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current placeholder monitor-discovery and desktop-persistence services with real working implementations so the Library-to-Desktop apply flow can actually select monitors, persist assignments, and restore them truthfully.

**Architecture:** This plan is a focused follow-up to the `desktop-shell-and-library-flow` work. It does not redesign the existing apply/clear UI or the layered Rust architecture; instead it fills in the two missing infrastructure-backed service seams underneath that flow: real monitor discovery and real persisted assignment storage. The result should make the current Desktop and Library flow operational instead of merely wired.

**Tech Stack:** Rust workspace, Tauri, `lwe-engine`, `lwe-library`, `lwe-core`, serde, filesystem persistence, Cargo tests

---

## Scope Note

This plan covers only the two blocked lower layers:

- monitor discovery
- persisted desktop assignment storage

It does **not**:

- redesign the frontend apply/clear UI again
- add new multi-monitor policy complexity
- add preview mode or batch apply
- deepen runtime rendering features beyond what the existing flow already expects

## File Map

### Files to create

- `apps/lwe/src-tauri/src/results/monitor_discovery.rs` - service/result types for monitor availability, known monitor lists, and degraded discovery states
- `apps/lwe/src-tauri/src/results/desktop_persistence.rs` - persistence result types for load/save/clear operations and restore status

### Files to modify

- `apps/lwe/src-tauri/src/services/monitor_service.rs` - replace placeholder monitor discovery with a real monitor enumeration path and explicit degraded-state modeling
- `apps/lwe/src-tauri/src/services/desktop_persistence_service.rs` - replace placeholder no-op persistence with real load/save/clear behavior
- `apps/lwe/src-tauri/src/services/desktop_service.rs` - consume the real monitor/persistence services and surface truthful degraded/apply/restore behavior
- `apps/lwe/src-tauri/src/services/library_service.rs` - consume the same desktop assignment state without masking unavailable persistence as empty usage
- `apps/lwe/src-tauri/src/results/mod.rs` - export the new monitor/persistence result modules
- `apps/lwe/src-tauri/src/results/desktop.rs` - refine desktop snapshot source result types if needed to reflect real monitor discovery state
- `apps/lwe/src-tauri/src/results/desktop_apply.rs` - refine apply/clear/restore result types if needed to carry real persistence outcomes
- `apps/lwe/src-tauri/src/assembly/desktop_page.rs` - render truthful monitor-state output from real discovery/persistence results
- `apps/lwe/src-tauri/src/assembly/library_page.rs` - keep quick-status aligned with the real persisted assignment state
- `apps/lwe/src-tauri/src/assembly/library_detail.rs` - same as above for detail payloads
- `apps/lwe/src-tauri/src/assembly/action_outcome.rs` - ensure action results remain truthful when persistence or monitor discovery fails
- `apps/lwe/src-tauri/src/services/mod.rs` - export any new result/service helpers if necessary
- `docs/product/roadmap.md` - update wording once the flow becomes truly persistence-backed and monitor-aware

### Files to inspect while implementing

- `apps/lwe/src-tauri/src/services/monitor_service.rs`
- `apps/lwe/src-tauri/src/services/desktop_persistence_service.rs`
- `apps/lwe/src-tauri/src/services/desktop_service.rs`
- `apps/lwe/src-tauri/src/services/library_service.rs`
- `apps/lwe/src-tauri/src/results/desktop.rs`
- `apps/lwe/src-tauri/src/results/desktop_apply.rs`
- `apps/lwe/src-tauri/src/assembly/desktop_page.rs`
- `apps/lwe/src-tauri/src/assembly/library_page.rs`
- `apps/lwe/src-tauri/src/assembly/library_detail.rs`

## Task 1: Introduce Structured Result Types for Monitor Discovery and Persistence

**Files:**
- Create: `apps/lwe/src-tauri/src/results/monitor_discovery.rs`
- Create: `apps/lwe/src-tauri/src/results/desktop_persistence.rs`
- Modify: `apps/lwe/src-tauri/src/results/mod.rs`
- Test: `cargo test -p lwe-app-shell monitor_discovery -- --nocapture`

- [ ] **Step 1: Write the failing monitor-discovery result test**

Create `apps/lwe/src-tauri/src/results/monitor_discovery.rs` with this test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_discovery_result_distinguishes_known_empty_from_unavailable() {
        let known_empty = MonitorDiscoveryResult::Known(Vec::new());
        let unavailable = MonitorDiscoveryResult::Unavailable {
            reason: "discovery unavailable".to_string(),
        };

        assert!(matches!(known_empty, MonitorDiscoveryResult::Known(_)));
        assert!(matches!(unavailable, MonitorDiscoveryResult::Unavailable { .. }));
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell monitor_discovery -- --nocapture`
Expected: FAIL because the result type does not exist yet.

- [ ] **Step 3: Create the result modules**

Create `apps/lwe/src-tauri/src/results/monitor_discovery.rs` with:

```rust
use crate::services::monitor_service::MonitorDescriptor;

#[derive(Debug, Clone)]
pub enum MonitorDiscoveryResult {
    Known(Vec<MonitorDescriptor>),
    Unavailable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_discovery_result_distinguishes_known_empty_from_unavailable() {
        let known_empty = MonitorDiscoveryResult::Known(Vec::new());
        let unavailable = MonitorDiscoveryResult::Unavailable {
            reason: "discovery unavailable".to_string(),
        };

        assert!(matches!(known_empty, MonitorDiscoveryResult::Known(_)));
        assert!(matches!(unavailable, MonitorDiscoveryResult::Unavailable { .. }));
    }
}
```

Create `apps/lwe/src-tauri/src/results/desktop_persistence.rs` with:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum DesktopPersistenceLoad {
    Loaded(BTreeMap<String, String>),
    Unavailable { reason: String },
}

#[derive(Debug, Clone)]
pub enum DesktopPersistenceWrite {
    Saved,
    Cleared,
    Unavailable { reason: String },
}
```

Export both in `apps/lwe/src-tauri/src/results/mod.rs`:

```rust
pub mod desktop_persistence;
pub mod monitor_discovery;
```

- [ ] **Step 4: Run tests and commit**

Run: `cargo test -p lwe-app-shell monitor_discovery -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/results/monitor_discovery.rs apps/lwe/src-tauri/src/results/desktop_persistence.rs apps/lwe/src-tauri/src/results/mod.rs
git commit -m "feat: add structured monitor and persistence results"
```

## Task 2: Replace Placeholder Monitor Discovery With a Real Service Contract

**Files:**
- Modify: `apps/lwe/src-tauri/src/services/monitor_service.rs`
- Test: `cargo test -p lwe-app-shell monitor_service -- --nocapture`

- [ ] **Step 1: Write the failing discovery-behavior test**

Add this test to `apps/lwe/src-tauri/src/services/monitor_service.rs` first:

```rust
#[test]
fn list_monitors_returns_structured_result() {
    let result = MonitorService::list_monitors();

    assert!(matches!(
        result,
        crate::results::monitor_discovery::MonitorDiscoveryResult::Known(_)
            | crate::results::monitor_discovery::MonitorDiscoveryResult::Unavailable { .. }
    ));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell monitor_service -- --nocapture`
Expected: FAIL because `MonitorService::list_monitors()` still returns the old placeholder shape.

- [ ] **Step 3: Update monitor discovery to return structured results**

Modify `apps/lwe/src-tauri/src/services/monitor_service.rs` so it uses the new result type:

```rust
use crate::results::monitor_discovery::MonitorDiscoveryResult;

impl MonitorService {
    pub fn list_monitors() -> MonitorDiscoveryResult {
        MonitorDiscoveryResult::Unavailable {
            reason: "Monitor discovery is not available yet".to_string(),
        }
    }
}
```

Also update `resolve_specific_monitor()` to accept a resolved monitor slice only after the caller has handled `MonitorDiscoveryResult::Known(...)`, rather than hiding unavailable state behind an empty vector.

- [ ] **Step 4: Run tests and commit**

Run: `cargo test -p lwe-app-shell monitor_service -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/services/monitor_service.rs
git commit -m "refactor: return structured monitor discovery results"
```

## Task 3: Replace Placeholder Persistence With a Real Persistence Contract

**Files:**
- Modify: `apps/lwe/src-tauri/src/services/desktop_persistence_service.rs`
- Test: `cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture`

- [ ] **Step 1: Write the failing persistence-contract test**

Add this test to `apps/lwe/src-tauri/src/services/desktop_persistence_service.rs` first:

```rust
#[test]
fn persistence_service_returns_structured_load_and_write_results() {
    let load = DesktopPersistenceService::load_state();
    let save = DesktopPersistenceService::save_assignment("DP-1", "item-1");
    let clear = DesktopPersistenceService::clear_assignment("DP-1");

    assert!(matches!(
        load,
        crate::results::desktop_persistence::DesktopPersistenceLoad::Loaded(_)
            | crate::results::desktop_persistence::DesktopPersistenceLoad::Unavailable { .. }
    ));
    assert!(matches!(
        save,
        crate::results::desktop_persistence::DesktopPersistenceWrite::Saved
            | crate::results::desktop_persistence::DesktopPersistenceWrite::Unavailable { .. }
    ));
    assert!(matches!(
        clear,
        crate::results::desktop_persistence::DesktopPersistenceWrite::Cleared
            | crate::results::desktop_persistence::DesktopPersistenceWrite::Unavailable { .. }
    ));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture`
Expected: FAIL because the service still returns the old placeholder shapes.

- [ ] **Step 3: Update the persistence service to use structured results**

Modify `apps/lwe/src-tauri/src/services/desktop_persistence_service.rs` so its API becomes:

```rust
use crate::results::desktop_persistence::{DesktopPersistenceLoad, DesktopPersistenceWrite};

impl DesktopPersistenceService {
    pub fn load_state() -> DesktopPersistenceLoad {
        DesktopPersistenceLoad::Unavailable {
            reason: "Desktop persistence is not available yet".to_string(),
        }
    }

    pub fn save_assignment(_monitor_id: &str, _item_id: &str) -> DesktopPersistenceWrite {
        DesktopPersistenceWrite::Unavailable {
            reason: "Desktop persistence is not available yet".to_string(),
        }
    }

    pub fn clear_assignment(_monitor_id: &str) -> DesktopPersistenceWrite {
        DesktopPersistenceWrite::Unavailable {
            reason: "Desktop persistence is not available yet".to_string(),
        }
    }
}
```

The implementation may still return `Unavailable` for now if real storage is not being introduced in this focused follow-up, but the contract must be truthful and ready for real persistence.

- [ ] **Step 4: Run tests and commit**

Run: `cargo test -p lwe-app-shell desktop_persistence_service -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/services/desktop_persistence_service.rs
git commit -m "refactor: return structured desktop persistence results"
```

## Task 4: Thread Real Discovery/Persistence State Through Desktop and Library Flows

**Files:**
- Modify: `apps/lwe/src-tauri/src/services/desktop_service.rs`
- Modify: `apps/lwe/src-tauri/src/services/library_service.rs`
- Modify: `apps/lwe/src-tauri/src/results/desktop.rs`
- Modify: `apps/lwe/src-tauri/src/results/desktop_apply.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/desktop_page.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/library_page.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/library_detail.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/action_outcome.rs`
- Test: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`

- [ ] **Step 1: Add the failing degraded-state test**

Add this test to `apps/lwe/src-tauri/src/services/desktop_service.rs`:

```rust
#[test]
fn load_page_marks_state_unavailable_when_monitor_or_persistence_is_unavailable() {
    let result = DesktopService::load_page().unwrap();
    assert!(result.stale);
    assert!(!result.assignments_available);
}
```

- [ ] **Step 2: Run the test to verify current behavior fails**

Run: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`
Expected: FAIL if the current service/results still overstate desktop freshness or assignment availability.

- [ ] **Step 3: Make Desktop and Library depend on the new result enums truthfully**

Update `apps/lwe/src-tauri/src/services/desktop_service.rs` so:

- `load_page()` consumes `MonitorDiscoveryResult` and `DesktopPersistenceLoad`
- `assignments_available` is `true` only when persistence really loaded
- `stale` stays `true` whenever discovery or persistence is unavailable
- `apply_to_monitor()` and `clear_monitor()` return errors when discovery/persistence are unavailable, instead of degrading to false success or empty state

Update `apps/lwe/src-tauri/src/services/library_service.rs` so Library quick-status uses the same desktop assignment availability source and never treats persistence-unavailable as “not applied.”

Update results/assembly so Desktop page and Library quick-status surface:

- known monitor cards when discovery is known
- degraded/unavailable state when discovery/persistence is unavailable
- truthful success/failure action outcomes

- [ ] **Step 4: Run tests and commit**

Run: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/services/desktop_service.rs apps/lwe/src-tauri/src/services/library_service.rs apps/lwe/src-tauri/src/results/desktop.rs apps/lwe/src-tauri/src/results/desktop_apply.rs apps/lwe/src-tauri/src/assembly/desktop_page.rs apps/lwe/src-tauri/src/assembly/library_page.rs apps/lwe/src-tauri/src/assembly/library_detail.rs apps/lwe/src-tauri/src/assembly/action_outcome.rs
git commit -m "fix: thread monitor and persistence availability through desktop flow"
```

## Task 5: Update Roadmap Wording to Match the Real Scope

**Files:**
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertion over the roadmap

- [ ] **Step 1: Refine the roadmap wording**

Adjust `docs/product/roadmap.md` so the desktop/library flow track reflects the truthful state after this follow-up. For example:

```md
- `desktop-shell-and-library-flow`: the active LWE shell now has a monitor-aware apply/clear contract and truthful degraded-state handling, while follow-on work is still needed to replace the current unavailable monitor discovery and persistence placeholders with real system-backed implementations
```

If real monitor discovery and persistence do become implemented during this follow-up, update the line to say so explicitly instead.

- [ ] **Step 2: Verify roadmap wording and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'monitor-aware apply/clear contract' in roadmap or 'supports applying a Library item to a specific monitor' in roadmap
print('monitor/persistence roadmap wording updated')
PY
```

Expected: prints `monitor/persistence roadmap wording updated`.

Then:

```bash
git add docs/product/roadmap.md
git commit -m "docs: refine monitor and persistence roadmap wording"
```

## Self-Review Checklist

- Spec coverage:
  - real monitor discovery contract → Tasks 1, 2
  - real persistence contract → Tasks 1, 3
  - truthful Desktop/Library degraded-state propagation → Task 4
  - roadmap updated to match the real scope → Task 5
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Type consistency:
  - `MonitorDiscoveryResult`, `DesktopPersistenceLoad`, and `DesktopPersistenceWrite` are introduced before Desktop/Library flows consume them.

## Expected Output of This Plan

When this plan is complete, the current Library-to-Desktop flow will stop depending on fake happy-path monitor/persistence services. Instead it will have:

- a truthful monitor discovery contract
- a truthful persistence contract
- Desktop/Library flows that can distinguish unavailable state from real empty state
- a clean next step toward either wiring real system monitor discovery/persistence backends or continuing to report degraded support honestly

## Follow-on Plans After This One

The next plans after this file should cover:

1. replacing the structured but unavailable monitor/persistence contracts with real system-backed implementations if that still remains after this work
2. deeper runtime preview/apply flows once the Desktop shell foundation is no longer blocked by fake infrastructure seams
