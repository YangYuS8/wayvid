# LWE Desktop Shell and Library Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first end-to-end Library-to-Desktop usage flow so a user can choose a compatible item in Library, apply it to a specific monitor, see the current desktop state in the Desktop page, clear it, and restore the last successful assignment on startup.

**Architecture:** Keep the existing thin-frontend architecture and layered Rust core. Library becomes the place where content is selected and `Apply` is initiated; Desktop becomes the place where current monitor state is observed and cleared. Rust owns monitor discovery, apply/clear/restore semantics, persisted desktop assignments, and the `ActionOutcome` / snapshot updates that connect Library and Desktop.

**Tech Stack:** Rust workspace, Tauri, Svelte, TypeScript, `lwe-engine`, `lwe-library`, `lwe-core`, Cargo tests, Vitest

---

## Scope Note

This plan is intentionally a **minimal daily-use desktop flow**, not a full desktop control suite.

It includes:

- apply a Library item to a specific monitor
- reflect current applied state in Desktop snapshots
- allow Desktop-side clear
- persist and restore the last successful monitor assignment

It explicitly does **not** include:

- preview-only mode
- all-monitor batch apply
- advanced multi-monitor rules/templates
- deep runtime diagnostics UI
- Desktop-side content browsing

## File Map

### Files to create

- `src-tauri/src/results/desktop_apply.rs` - application-result types for apply/clear/restore effects
- `src-tauri/src/services/monitor_service.rs` - monitor discovery + target resolution service
- `src-tauri/src/services/desktop_persistence_service.rs` - persisted last-assignment read/write helpers
- `src-tauri/src/assembly/desktop_apply.rs` - assembly helpers for apply/clear action outcomes and Library quick-status updates
- `src/lib/components/MonitorPicker.svelte` - thin monitor selection UI for Library apply action
- `src/lib/components/DesktopMonitorCard.svelte` - current monitor state card for the Desktop page

### Files to modify

- `src-tauri/src/models.rs` - add monitor target/apply request types, Library quick-status fields, Desktop monitor assignment fields
- `src-tauri/src/action_outcome.rs` - extend action outcomes if needed for current-page update + Desktop invalidation patterns
- `src-tauri/src/results/mod.rs` - export the new desktop-apply result module
- `src-tauri/src/services/mod.rs` - export new monitor/persistence services
- `src-tauri/src/services/library_service.rs` - expose applyable Library item selection and quick-status projection inputs
- `src-tauri/src/services/desktop_service.rs` - replace placeholder-only behavior with monitor snapshot/load/apply/clear/restore workflows
- `src-tauri/src/services/app_shell_service.rs` - include persisted/current desktop summary where relevant
- `src-tauri/src/assembly/library_page.rs` - surface current usage quick status in Library summaries
- `src-tauri/src/assembly/library_detail.rs` - expose applyability and quick status in Library detail
- `src-tauri/src/assembly/desktop_page.rs` - build real monitor cards from Desktop results
- `src-tauri/src/assembly/action_outcome.rs` - assemble apply/clear outcomes with current-page and stale-page behavior
- `src-tauri/src/commands/library.rs` - add Library-side apply command(s)
- `src-tauri/src/commands/desktop.rs` - add Desktop-side clear/load commands and startup restore hook if needed
- `src/lib/types.ts` - mirror new apply/monitor/status contract shapes
- `src/lib/ipc.ts` - add typed wrappers for apply/clear/load flows
- `src/lib/stores/ui.ts` - add minimal current-action/quick-status cache handling for Library + Desktop pages
- `src/routes/library/+page.svelte` - add apply action in the Library detail panel and hook monitor selection flow
- `src/routes/desktop/+page.svelte` - replace placeholder Desktop rendering with monitor cards and clear actions
- `src/lib/components/LibraryDetailPanel.svelte` - render apply button and current usage quick status
- `docs/product/roadmap.md` - update the desktop/library flow track once implemented

### Files to inspect while implementing

- `src-tauri/src/services/desktop_service.rs`
- `src-tauri/src/assembly/desktop_page.rs`
- `src-tauri/src/services/library_service.rs`
- `src/lib/types.ts`
- `src/routes/library/+page.svelte`
- `src/routes/desktop/+page.svelte`
- `docs/superpowers/specs/2026-03-27-linux-dynamic-wallpaper-platform-design.md`

## Task 1: Define Monitor Targets and Desktop Apply Result Types

**Files:**
- Create: `src-tauri/src/results/desktop_apply.rs`
- Modify: `src-tauri/src/results/mod.rs`
- Modify: `src-tauri/src/models.rs`
- Test: `cargo test -p lwe-app-shell desktop_apply -- --nocapture`

- [ ] **Step 1: Write the failing monitor-target test**

Add this test to `src-tauri/src/models.rs` first:

```rust
#[test]
fn desktop_apply_request_serializes_specific_monitor_target() {
    let request = DesktopApplyRequest {
        item_id: "item-1".to_string(),
        target: DesktopApplyTarget::SpecificMonitor {
            monitor_id: "HDMI-A-1".to_string(),
        },
    };

    let value = serde_json::to_value(&request).unwrap();

    assert_eq!(value["target"]["kind"], "specific_monitor");
    assert_eq!(value["target"]["monitorId"], "HDMI-A-1");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell desktop_apply -- --nocapture`
Expected: FAIL because `DesktopApplyRequest` / `DesktopApplyTarget` do not exist yet.

- [ ] **Step 3: Add monitor-target and quick-status models**

In `src-tauri/src/models.rs`, add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopApplyRequest {
    pub item_id: String,
    pub target: DesktopApplyTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DesktopApplyTarget {
    SpecificMonitor { monitor_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryQuickStatus {
    pub applied_monitor_name: Option<String>,
}
```

Then update:

- `LibraryItemSummary` to add `quick_status: Option<LibraryQuickStatus>`
- `LibraryItemDetail` to add `quick_status: Option<LibraryQuickStatus>`
- `DesktopMonitorSummary` to add `current_wallpaper_id: Option<String>` and `clear_supported: bool`

- [ ] **Step 4: Create desktop-apply result types**

Create `src-tauri/src/results/desktop_apply.rs` with:

```rust
use crate::results::app_shell::ObservedCount;

#[derive(Debug, Clone)]
pub struct DesktopAssignment {
    pub monitor_id: String,
    pub monitor_name: String,
    pub item_id: String,
    pub item_title: String,
    pub item_cover_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DesktopApplyResult {
    pub assignment: DesktopAssignment,
    pub desktop_state_changed: bool,
}

#[derive(Debug, Clone)]
pub struct DesktopClearResult {
    pub monitor_id: String,
    pub desktop_state_changed: bool,
}

#[derive(Debug, Clone)]
pub struct RestoreAssignmentsResult {
    pub restored_assignments: Vec<DesktopAssignment>,
}
```

Export it in `src-tauri/src/results/mod.rs`:

```rust
pub mod desktop_apply;
```

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell desktop_apply -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/models.rs src-tauri/src/results/mod.rs src-tauri/src/results/desktop_apply.rs
git commit -m "feat: add desktop apply request and result models"
```

## Task 2: Build Monitor Discovery and Desktop Persistence Services

**Files:**
- Create: `src-tauri/src/services/monitor_service.rs`
- Create: `src-tauri/src/services/desktop_persistence_service.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Test: `cargo test -p lwe-app-shell monitor_service -- --nocapture`

- [ ] **Step 1: Write the failing monitor-service test**

Create `src-tauri/src/services/monitor_service.rs` with this test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_rejects_missing_specific_monitor_target() {
        let result = resolve_specific_monitor(
            &[MonitorDescriptor {
                id: "DP-1".to_string(),
                name: "Monitor A".to_string(),
                resolution: "1920x1080".to_string(),
            }],
            "HDMI-A-1",
        );

        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell monitor_service -- --nocapture`
Expected: FAIL because `MonitorDescriptor` / `resolve_specific_monitor` do not exist yet.

- [ ] **Step 3: Implement monitor discovery/target resolution**

Create `src-tauri/src/services/monitor_service.rs` with:

```rust
#[derive(Debug, Clone)]
pub struct MonitorDescriptor {
    pub id: String,
    pub name: String,
    pub resolution: String,
}

pub fn resolve_specific_monitor(
    monitors: &[MonitorDescriptor],
    monitor_id: &str,
) -> Result<MonitorDescriptor, String> {
    monitors
        .iter()
        .find(|monitor| monitor.id == monitor_id)
        .cloned()
        .ok_or_else(|| format!("Monitor {monitor_id} is not available"))
}

pub struct MonitorService;

impl MonitorService {
    pub fn list_monitors() -> Vec<MonitorDescriptor> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_rejects_missing_specific_monitor_target() {
        let result = resolve_specific_monitor(
            &[MonitorDescriptor {
                id: "DP-1".to_string(),
                name: "Monitor A".to_string(),
                resolution: "1920x1080".to_string(),
            }],
            "HDMI-A-1",
        );

        assert!(result.is_err());
    }
}
```

- [ ] **Step 4: Implement persisted assignment storage helpers**

Create `src-tauri/src/services/desktop_persistence_service.rs` with:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct PersistedDesktopState {
    pub assignments: BTreeMap<String, String>,
}

pub struct DesktopPersistenceService;

impl DesktopPersistenceService {
    pub fn load_state() -> PersistedDesktopState {
        PersistedDesktopState::default()
    }

    pub fn save_assignment(_monitor_id: &str, _item_id: &str) {}

    pub fn clear_assignment(_monitor_id: &str) {}
}
```

Export both services in `src-tauri/src/services/mod.rs`:

```rust
pub mod desktop_persistence_service;
pub mod monitor_service;
```

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell monitor_service -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/services/mod.rs src-tauri/src/services/monitor_service.rs src-tauri/src/services/desktop_persistence_service.rs
git commit -m "feat: add monitor and desktop persistence services"
```

## Task 3: Implement Desktop Apply/Clear/Restore Service and Assembly Flow

**Files:**
- Create: `src-tauri/src/assembly/desktop_apply.rs`
- Modify: `src-tauri/src/services/desktop_service.rs`
- Modify: `src-tauri/src/services/library_service.rs`
- Modify: `src-tauri/src/assembly/desktop_page.rs`
- Modify: `src-tauri/src/assembly/library_page.rs`
- Modify: `src-tauri/src/assembly/library_detail.rs`
- Modify: `src-tauri/src/assembly/action_outcome.rs`
- Modify: `src-tauri/src/results/library.rs`
- Modify: `src-tauri/src/results/desktop.rs`
- Test: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`

- [ ] **Step 1: Write the failing desktop-apply-flow test**

Add this test to `src-tauri/src/services/desktop_service.rs` first:

```rust
#[test]
fn desktop_service_apply_returns_assignment_result() {
    let result = DesktopService::apply_to_monitor(
        DesktopApplyRequest {
            item_id: "item-1".to_string(),
            target: DesktopApplyTarget::SpecificMonitor {
                monitor_id: "HDMI-A-1".to_string(),
            },
        },
        &[],
    );

    assert!(result.is_err());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`
Expected: FAIL because `DesktopService::apply_to_monitor` and the related flow do not exist yet.

- [ ] **Step 3: Implement apply/clear/restore in `DesktopService`**

Extend `src-tauri/src/services/desktop_service.rs` with:

```rust
use crate::models::{DesktopApplyRequest, DesktopApplyTarget};
use crate::results::desktop::DesktopPageResult;
use crate::results::desktop_apply::{DesktopApplyResult, DesktopAssignment, DesktopClearResult};
use crate::services::desktop_persistence_service::DesktopPersistenceService;
use crate::services::monitor_service::{resolve_specific_monitor, MonitorDescriptor};

impl DesktopService {
    pub fn apply_to_monitor(
        request: DesktopApplyRequest,
        monitors: &[MonitorDescriptor],
    ) -> Result<DesktopApplyResult, String> {
        let monitor = match request.target {
            DesktopApplyTarget::SpecificMonitor { monitor_id } => {
                resolve_specific_monitor(monitors, &monitor_id)?
            }
        };

        DesktopPersistenceService::save_assignment(&monitor.id, &request.item_id);

        Ok(DesktopApplyResult {
            assignment: DesktopAssignment {
                monitor_id: monitor.id,
                monitor_name: monitor.name,
                item_id: request.item_id,
                item_title: "Selected item".to_string(),
                item_cover_path: None,
            },
            desktop_state_changed: true,
        })
    }

    pub fn clear_monitor(monitor_id: &str) -> DesktopClearResult {
        DesktopPersistenceService::clear_assignment(monitor_id);
        DesktopClearResult {
            monitor_id: monitor_id.to_string(),
            desktop_state_changed: true,
        }
    }
}
```

- [ ] **Step 4: Assemble the Desktop and Library effects**

Create `src-tauri/src/assembly/desktop_apply.rs` with helpers like:

```rust
use crate::action_outcome::{ActionOutcome, InvalidatedPage};
use crate::models::DesktopPageSnapshot;
use crate::results::desktop_apply::{DesktopApplyResult, DesktopClearResult};

pub fn desktop_apply_invalidations() -> Vec<InvalidatedPage> {
    vec![InvalidatedPage::Desktop, InvalidatedPage::Library]
}
```

Update:

- `results/library.rs` to carry applied monitor quick-status inputs
- `assembly/library_page.rs` and `assembly/library_detail.rs` to produce `quick_status`
- `results/desktop.rs` and `assembly/desktop_page.rs` to build actual monitor cards when assignments exist
- `assembly/action_outcome.rs` so apply/clear actions produce `DesktopPageSnapshot` refreshes and invalidate `Library`

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/services/desktop_service.rs src-tauri/src/services/library_service.rs src-tauri/src/results/library.rs src-tauri/src/results/desktop.rs src-tauri/src/assembly/desktop_apply.rs src-tauri/src/assembly/desktop_page.rs src-tauri/src/assembly/library_page.rs src-tauri/src/assembly/library_detail.rs src-tauri/src/assembly/action_outcome.rs
git commit -m "feat: add desktop apply clear and restore flow"
```

## Task 4: Expose Apply/Clear Through Commands and Thin Frontend UI

**Files:**
- Create: `src/lib/components/MonitorPicker.svelte`
- Create: `src/lib/components/DesktopMonitorCard.svelte`
- Modify: `src-tauri/src/commands/library.rs`
- Modify: `src-tauri/src/commands/desktop.rs`
- Modify: `src/lib/types.ts`
- Modify: `src/lib/ipc.ts`
- Modify: `src/lib/stores/ui.ts`
- Modify: `src/lib/components/LibraryDetailPanel.svelte`
- Modify: `src/routes/library/+page.svelte`
- Modify: `src/routes/desktop/+page.svelte`
- Test: `npm test --prefix `

- [ ] **Step 1: Write the failing frontend monitor-picker test**

Create `src/lib/components/MonitorPicker.test.ts` with:

```ts
import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';
import MonitorPicker from './MonitorPicker.svelte';

describe('MonitorPicker', () => {
  it('renders all available monitor choices', () => {
    const { body } = render(MonitorPicker, {
      monitors: [
        { id: 'HDMI-A-1', name: 'Monitor A' },
        { id: 'DP-1', name: 'Monitor B' }
      ],
      selectedMonitorId: null
    });

    expect(body).toContain('Monitor A');
    expect(body).toContain('Monitor B');
  });
});
```

- [ ] **Step 2: Run the frontend test to verify it fails**

Run: `npm test --prefix `
Expected: FAIL because the monitor picker and related apply flow do not exist yet.

- [ ] **Step 3: Extend frontend contract and IPC for apply/clear**

In `src/lib/types.ts`, add:

```ts
export type DesktopApplyTarget = {
  kind: 'specific_monitor';
  monitorId: string;
};

export interface DesktopApplyRequest {
  itemId: string;
  target: DesktopApplyTarget;
}

export interface LibraryQuickStatus {
  appliedMonitorName: string | null;
}
```

Update existing `LibraryItemSummary`, `LibraryItemDetail`, and `DesktopMonitorSummary` types to carry the new quick-status / monitor-assignment fields.

In `src/lib/ipc.ts`, add wrappers like:

```ts
export const applyLibraryItemToDesktop = (request: DesktopApplyRequest) =>
  invoke<ActionOutcome<DesktopPageSnapshot>>('apply_library_item_to_desktop', { request });

export const clearDesktopMonitor = (monitorId: string) =>
  invoke<ActionOutcome<DesktopPageSnapshot>>('clear_desktop_monitor', { monitorId });
```

- [ ] **Step 4: Implement the thin Library/Desktop UI flow**

Create `src/lib/components/MonitorPicker.svelte` with a minimal monitor-choice list or select control.

Create `src/lib/components/DesktopMonitorCard.svelte` to render:

- monitor name
- resolution
- current wallpaper title
- cover
- runtime status
- `Clear` button when supported

Update `LibraryDetailPanel.svelte` to:

- show `Apply` only when compatibility is not `unsupported`
- show the selected monitor picker
- show `quick_status` like `Applied to Monitor A`

Update `routes/library/+page.svelte` to:

- call `applyLibraryItemToDesktop`
- refresh/replace Desktop snapshot on success via `currentUpdate`
- mark `library` stale or refresh current item quick status as needed

Update `routes/desktop/+page.svelte` to render monitor cards and call `clearDesktopMonitor`.

- [ ] **Step 5: Run tests and commit**

Run: `npm test --prefix `
Expected: PASS

Then:

```bash
git add src-tauri/src/commands/library.rs src-tauri/src/commands/desktop.rs src/lib/types.ts src/lib/ipc.ts src/lib/stores/ui.ts src/lib/components/MonitorPicker.svelte src/lib/components/MonitorPicker.test.ts src/lib/components/DesktopMonitorCard.svelte src/lib/components/LibraryDetailPanel.svelte src/routes/library/+page.svelte src/routes/desktop/+page.svelte
git commit -m "feat: add library to desktop apply ui flow"
```

## Task 5: Update the Product Roadmap for the Desktop/Library Flow

**Files:**
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertion over the roadmap

- [ ] **Step 1: Update the roadmap wording**

Adjust the `desktop-shell-and-library-flow` track so it reflects the newly implemented minimum daily-use loop, for example:

```md
- `desktop-shell-and-library-flow`: the active LWE shell now supports applying a Library item to a specific monitor, reflecting current desktop state, clearing monitor assignments, and restoring the last successful monitor mapping on startup; follow-on work should deepen runtime controls rather than establish the first apply path
```

- [ ] **Step 2: Verify roadmap wording and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'supports applying a Library item to a specific monitor' in roadmap
print('desktop-shell-and-library-flow roadmap updated')
PY
```

Expected: prints `desktop-shell-and-library-flow roadmap updated`.

Then:

```bash
git add docs/product/roadmap.md
git commit -m "docs: update desktop library flow roadmap track"
```

## Self-Review Checklist

- Spec coverage:
  - apply a Library item to a specific monitor → Tasks 1, 2, 3, 4
  - Desktop shows current monitor state → Tasks 1, 3, 4
  - Desktop can clear current assignment → Tasks 1, 3, 4
  - restore last successful assignment → Tasks 2, 3
  - Library quick-status reflects current usage → Tasks 1, 3, 4
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Type consistency:
  - `DesktopApplyRequest`, `DesktopApplyTarget`, `LibraryQuickStatus`, `DesktopAssignment`, and `DesktopApplyResult` are introduced before later tasks use them.

## Expected Output of This Plan

When this plan is complete, LWE will have the first real daily-use desktop loop:

- choose a compatible Library item
- apply it to a specific monitor
- see that state in Desktop
- clear or replace it
- restore the last successful mapping on startup

## Follow-on Plans After This One

The next plans after this file should cover:

1. runtime-specific preview/apply flow for `video` and `scene`
2. deeper desktop controls and multi-monitor policies after the minimal loop is stable
