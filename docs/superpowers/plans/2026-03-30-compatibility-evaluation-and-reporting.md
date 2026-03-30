# LWE Compatibility Evaluation and Reporting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a real compatibility evaluation and reporting pipeline for LWE so Workshop and Library surfaces show structured compatibility levels, reasoned explanations, and next-step guidance instead of hard-coded badge strings and ad hoc notes.

**Architecture:** This plan extends the layered `lwe-app-shell` core without changing the thin-frontend contract shape. Compatibility moves from scattered per-entry mapping into a dedicated evaluation path: policies define the rules, services produce application-level compatibility results, assemblers translate those into summary/detail payloads, and the frontend renders richer status + explanation blocks without taking ownership of business logic.

**Tech Stack:** Rust workspace, Tauri, Svelte, TypeScript, `lwe-library`, `lwe-core`, Cargo tests, Vitest

---

## Scope Note

This plan focuses on **compatibility evaluation and user-facing reporting**, not runtime execution changes.

It does **not**:

- add new `video` or `scene` runtime capabilities
- make `web` wallpapers runnable
- redesign page structure or the thin frontend cache model
- change the retained-core crate boundaries again

## File Map

### Files to create

- `apps/lwe/src-tauri/src/results/compatibility.rs` - application-result types for compatibility assessments and explanation data
- `apps/lwe/src-tauri/src/services/compatibility_service.rs` - service entrypoints that evaluate compatibility for Workshop entries and Library projections
- `apps/lwe/src-tauri/src/assembly/compatibility.rs` - helpers that translate compatibility results into frontend-facing summary/detail fields
- `apps/lwe/src/lib/components/CompatibilityPanel.svelte` - shared frontend component for explanation blocks and next-step guidance

### Files to modify

- `apps/lwe/src-tauri/src/policies/shared/compatibility_policy.rs` - replace note-string generation with structured reason/evidence outputs
- `apps/lwe/src-tauri/src/policies/shared/support_policy.rs` - expose any helpers needed by compatibility evaluation
- `apps/lwe/src-tauri/src/services/mod.rs` - export compatibility service
- `apps/lwe/src-tauri/src/results/mod.rs` - export compatibility results
- `apps/lwe/src-tauri/src/assembly/mod.rs` - export compatibility assembly helpers
- `apps/lwe/src-tauri/src/models.rs` - add structured compatibility explanation fields to Workshop/Library detail payloads and refined summary badge data if needed
- `apps/lwe/src-tauri/src/assembly/workshop_page.rs` - use compatibility assembly helpers for summaries
- `apps/lwe/src-tauri/src/assembly/workshop_detail.rs` - assemble structured compatibility detail instead of hard-coded note strings
- `apps/lwe/src-tauri/src/assembly/library_page.rs` - include compatibility summary for imported Library items where appropriate
- `apps/lwe/src-tauri/src/assembly/library_detail.rs` - expose compatibility explanation for Library detail payloads
- `apps/lwe/src-tauri/src/services/workshop_service.rs` - feed Workshop refresh/inspect flows through compatibility evaluation
- `apps/lwe/src-tauri/src/services/library_service.rs` - evaluate compatibility for Workshop-projected library items
- `apps/lwe/src/lib/types.ts` - add matching TypeScript compatibility explanation/result types
- `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte` - render structured compatibility information via `CompatibilityPanel`
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte` - render the same compatibility explanation model
- `apps/lwe/src/routes/workshop/+page.svelte` - expose any new detail fields cleanly
- `apps/lwe/src/routes/library/+page.svelte` - expose any new detail fields cleanly
- `docs/product/roadmap.md` - mark the compatibility track more concretely once implemented

### Files to inspect while implementing

- `apps/lwe/src-tauri/src/policies/shared/compatibility_policy.rs`
- `apps/lwe/src-tauri/src/results/workshop.rs`
- `apps/lwe/src-tauri/src/results/library.rs`
- `apps/lwe/src-tauri/src/assembly/workshop_detail.rs`
- `apps/lwe/src-tauri/src/assembly/library_detail.rs`
- `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte`
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- `docs/superpowers/specs/2026-03-27-linux-dynamic-wallpaper-platform-design.md`

## Task 1: Replace Compatibility Strings With Structured Evaluation Results

**Files:**
- Create: `apps/lwe/src-tauri/src/results/compatibility.rs`
- Modify: `apps/lwe/src-tauri/src/policies/shared/compatibility_policy.rs`
- Modify: `apps/lwe/src-tauri/src/results/mod.rs`
- Test: `cargo test -p lwe-app-shell compatibility_policy -- --nocapture`

- [ ] **Step 1: Write the failing structured-compatibility test**

Add this test to `apps/lwe/src-tauri/src/policies/shared/compatibility_policy.rs` first:

```rust
#[test]
fn compatibility_decision_exposes_structured_reason_and_guidance() {
    let entry = WorkshopCatalogEntry {
        workshop_id: 9,
        title: "Broken Scene".to_string(),
        project_type: WorkshopProjectType::Scene,
        project_dir: std::path::PathBuf::from("/tmp/9"),
        cover_path: None,
        sync_state: WorkshopSyncState::MissingPrimaryAsset,
        supported_first_release: false,
        library_item_id: None,
    };

    let decision = compatibility_decision(&entry);

    assert_eq!(decision.level, CompatibilityLevel::PartiallySupported);
    assert_eq!(decision.reason, CompatibilityReason::MissingPrimaryAsset);
    assert_eq!(decision.next_step, CompatibilityNextStep::ResyncWorkshopItem);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell compatibility_policy -- --nocapture`
Expected: FAIL because `CompatibilityNextStep` and structured compatibility result fields do not exist yet.

- [ ] **Step 3: Create compatibility result types**

Create `apps/lwe/src-tauri/src/results/compatibility.rs` with:

```rust
use serde::{Deserialize, Serialize};

use crate::models::CompatibilityBadge;
use crate::policies::shared::compatibility_policy::{CompatibilityLevel, CompatibilityReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityNextStep {
    None,
    OpenInSteam,
    ResyncWorkshopItem,
    WaitForFutureSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatibilityAssessment {
    pub level: CompatibilityLevel,
    pub reason: CompatibilityReason,
    pub next_step: CompatibilityNextStep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilitySummary {
    pub badge: CompatibilityBadge,
    pub reason_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityExplanation {
    pub badge: CompatibilityBadge,
    pub reason_code: String,
    pub headline: String,
    pub detail: String,
    pub next_step: CompatibilityNextStep,
}
```

- [ ] **Step 4: Update the shared compatibility policy to return structured decisions**

In `apps/lwe/src-tauri/src/policies/shared/compatibility_policy.rs`, change `CompatibilityDecision` so it becomes:

```rust
pub struct CompatibilityDecision {
    pub level: CompatibilityLevel,
    pub reason: CompatibilityReason,
    pub next_step: crate::results::compatibility::CompatibilityNextStep,
}
```

Then update `compatibility_decision()` to set `next_step` explicitly:

```rust
CompatibilityReason::ReadyForLibrary => CompatibilityNextStep::None
CompatibilityReason::MissingProjectMetadata => CompatibilityNextStep::ResyncWorkshopItem
CompatibilityReason::MissingPrimaryAsset => CompatibilityNextStep::ResyncWorkshopItem
CompatibilityReason::UnsupportedWebItem => CompatibilityNextStep::WaitForFutureSupport
CompatibilityReason::UnsupportedProjectType => CompatibilityNextStep::WaitForFutureSupport
```

Export the new result module in `apps/lwe/src-tauri/src/results/mod.rs`:

```rust
pub mod compatibility;
```

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell compatibility_policy -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/results/compatibility.rs apps/lwe/src-tauri/src/results/mod.rs apps/lwe/src-tauri/src/policies/shared/compatibility_policy.rs
git commit -m "feat: add structured compatibility assessments"
```

## Task 2: Route Workshop and Library Through a Compatibility Service

**Files:**
- Create: `apps/lwe/src-tauri/src/services/compatibility_service.rs`
- Modify: `apps/lwe/src-tauri/src/services/mod.rs`
- Modify: `apps/lwe/src-tauri/src/services/workshop_service.rs`
- Modify: `apps/lwe/src-tauri/src/services/library_service.rs`
- Test: `cargo test -p lwe-app-shell compatibility_service -- --nocapture`

- [ ] **Step 1: Write the failing compatibility-service test**

Create `apps/lwe/src-tauri/src/services/compatibility_service.rs` with this test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lwe_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

    #[test]
    fn service_evaluates_catalog_entry_into_assessment() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 1,
            title: "Video".to_string(),
            project_type: WorkshopProjectType::Video,
            project_dir: std::path::PathBuf::from("/tmp/1"),
            cover_path: None,
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("video-1".to_string()),
        };

        let assessment = CompatibilityService::assess_workshop_entry(&entry);
        assert_eq!(assessment.reason, crate::policies::shared::compatibility_policy::CompatibilityReason::ReadyForLibrary);
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell compatibility_service -- --nocapture`
Expected: FAIL because `CompatibilityService` does not exist yet.

- [ ] **Step 3: Implement the compatibility service**

Create `apps/lwe/src-tauri/src/services/compatibility_service.rs` with:

```rust
use crate::policies::shared::compatibility_policy::compatibility_decision;
use crate::results::compatibility::CompatibilityAssessment;
use lwe_library::WorkshopCatalogEntry;

pub struct CompatibilityService;

impl CompatibilityService {
    pub fn assess_workshop_entry(entry: &WorkshopCatalogEntry) -> CompatibilityAssessment {
        let decision = compatibility_decision(entry);
        CompatibilityAssessment {
            level: decision.level,
            reason: decision.reason,
            next_step: decision.next_step,
        }
    }

    pub fn assess_workshop_entries(entries: &[WorkshopCatalogEntry]) -> Vec<(WorkshopCatalogEntry, CompatibilityAssessment)> {
        entries
            .iter()
            .cloned()
            .map(|entry| {
                let assessment = Self::assess_workshop_entry(&entry);
                (entry, assessment)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lwe_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

    #[test]
    fn service_evaluates_catalog_entry_into_assessment() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 1,
            title: "Video".to_string(),
            project_type: WorkshopProjectType::Video,
            project_dir: std::path::PathBuf::from("/tmp/1"),
            cover_path: None,
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("video-1".to_string()),
        };

        let assessment = CompatibilityService::assess_workshop_entry(&entry);
        assert_eq!(assessment.reason, crate::policies::shared::compatibility_policy::CompatibilityReason::ReadyForLibrary);
    }
}
```

- [ ] **Step 4: Feed Workshop and Library service results through compatibility evaluation**

In `apps/lwe/src-tauri/src/services/mod.rs`, add:

```rust
pub mod compatibility_service;
```

In `apps/lwe/src-tauri/src/services/workshop_service.rs`, extend `WorkshopRefreshResult` / `WorkshopInspection` population to include compatibility assessments, for example by building a vector of `(WorkshopCatalogEntry, CompatibilityAssessment)` pairs instead of plain entries.

In `apps/lwe/src-tauri/src/services/library_service.rs`, use the same service to evaluate the entries that are projected into Library so the later assemblers can attach compatibility summaries without recomputing policy logic.

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell compatibility_service -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/services/mod.rs apps/lwe/src-tauri/src/services/compatibility_service.rs apps/lwe/src-tauri/src/services/workshop_service.rs apps/lwe/src-tauri/src/services/library_service.rs
git commit -m "refactor: route lwe compatibility through service layer"
```

## Task 3: Assemble Compatibility Summaries and Explanations Into Frontend Models

**Files:**
- Create: `apps/lwe/src-tauri/src/assembly/compatibility.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/mod.rs`
- Modify: `apps/lwe/src-tauri/src/models.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/workshop_page.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/workshop_detail.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/library_page.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/library_detail.rs`
- Test: `cargo test -p lwe-app-shell compatibility_assembly -- --nocapture`

- [ ] **Step 1: Write the failing compatibility-assembly test**

Create `apps/lwe/src-tauri/src/assembly/compatibility.rs` with this test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::policies::shared::compatibility_policy::{CompatibilityLevel, CompatibilityReason};
    use crate::results::compatibility::{CompatibilityAssessment, CompatibilityNextStep};

    #[test]
    fn assembly_turns_assessment_into_summary_and_explanation() {
        let assessment = CompatibilityAssessment {
            level: CompatibilityLevel::Unsupported,
            reason: CompatibilityReason::UnsupportedWebItem,
            next_step: CompatibilityNextStep::WaitForFutureSupport,
        };

        let summary = compatibility_summary(&assessment);
        let explanation = compatibility_explanation(&assessment);

        assert_eq!(summary.reason_code, "unsupported_web_item");
        assert_eq!(explanation.next_step, CompatibilityNextStep::WaitForFutureSupport);
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell compatibility_assembly -- --nocapture`
Expected: FAIL because the new compatibility assembly helpers do not exist yet.

- [ ] **Step 3: Add frontend-facing compatibility types to the models**

In `apps/lwe/src-tauri/src/models.rs`, add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilitySummaryModel {
    pub badge: CompatibilityBadge,
    pub reason_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityExplanationModel {
    pub badge: CompatibilityBadge,
    pub reason_code: String,
    pub headline: String,
    pub detail: String,
    pub next_step: crate::results::compatibility::CompatibilityNextStep,
}
```

Then update:

- `WorkshopItemSummary` to replace `compatibility_badge` with `compatibility: CompatibilitySummaryModel`
- `WorkshopItemDetail` to replace `compatibility_badge` / `compatibility_note` with `compatibility: CompatibilityExplanationModel`
- `LibraryItemSummary` to add `compatibility: CompatibilitySummaryModel`
- `LibraryItemDetail` to add `compatibility: CompatibilityExplanationModel`

- [ ] **Step 4: Implement compatibility assembly helpers and wire page/detail assemblers**

Create `apps/lwe/src-tauri/src/assembly/compatibility.rs` with:

```rust
use crate::models::{CompatibilityBadge, CompatibilityExplanationModel, CompatibilitySummaryModel};
use crate::policies::shared::compatibility_policy::{CompatibilityLevel, CompatibilityReason};
use crate::results::compatibility::{CompatibilityAssessment, CompatibilityNextStep};

fn badge(level: CompatibilityLevel) -> CompatibilityBadge {
    match level {
        CompatibilityLevel::FullySupported => CompatibilityBadge::FullySupported,
        CompatibilityLevel::PartiallySupported => CompatibilityBadge::PartiallySupported,
        CompatibilityLevel::Unsupported => CompatibilityBadge::Unsupported,
    }
}

fn reason_code(reason: CompatibilityReason) -> String {
    match reason {
        CompatibilityReason::ReadyForLibrary => "ready_for_library".to_string(),
        CompatibilityReason::MissingProjectMetadata => "missing_project_metadata".to_string(),
        CompatibilityReason::MissingPrimaryAsset => "missing_primary_asset".to_string(),
        CompatibilityReason::UnsupportedWebItem => "unsupported_web_item".to_string(),
        CompatibilityReason::UnsupportedProjectType => "unsupported_project_type".to_string(),
    }
}

pub fn compatibility_summary(assessment: &CompatibilityAssessment) -> CompatibilitySummaryModel {
    CompatibilitySummaryModel {
        badge: badge(assessment.level),
        reason_code: reason_code(assessment.reason),
    }
}

pub fn compatibility_explanation(assessment: &CompatibilityAssessment) -> CompatibilityExplanationModel {
    let (headline, detail) = match assessment.reason {
        CompatibilityReason::ReadyForLibrary => (
            "Ready to use".to_string(),
            "This item is synchronized locally and available for Library and desktop use.".to_string(),
        ),
        CompatibilityReason::MissingProjectMetadata => (
            "Missing project metadata".to_string(),
            "LWE found the local Workshop folder, but the project metadata is missing or unreadable.".to_string(),
        ),
        CompatibilityReason::MissingPrimaryAsset => (
            "Missing primary asset".to_string(),
            "The project metadata exists, but the primary video or scene asset is missing from the local Workshop item.".to_string(),
        ),
        CompatibilityReason::UnsupportedWebItem => (
            "Web item not in first release".to_string(),
            "Web Workshop items are recognized, but LWE first-release support is currently limited to video and scene wallpapers.".to_string(),
        ),
        CompatibilityReason::UnsupportedProjectType => (
            "Project type not supported".to_string(),
            "This Workshop item uses a project type outside the current first-release import surface.".to_string(),
        ),
    };

    CompatibilityExplanationModel {
        badge: badge(assessment.level),
        reason_code: reason_code(assessment.reason),
        headline,
        detail,
        next_step: assessment.next_step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policies::shared::compatibility_policy::{CompatibilityLevel, CompatibilityReason};
    use crate::results::compatibility::{CompatibilityAssessment, CompatibilityNextStep};

    #[test]
    fn assembly_turns_assessment_into_summary_and_explanation() {
        let assessment = CompatibilityAssessment {
            level: CompatibilityLevel::Unsupported,
            reason: CompatibilityReason::UnsupportedWebItem,
            next_step: CompatibilityNextStep::WaitForFutureSupport,
        };

        let summary = compatibility_summary(&assessment);
        let explanation = compatibility_explanation(&assessment);

        assert_eq!(summary.reason_code, "unsupported_web_item");
        assert_eq!(explanation.next_step, CompatibilityNextStep::WaitForFutureSupport);
    }
}
```

Export it from `apps/lwe/src-tauri/src/assembly/mod.rs`:

```rust
pub mod compatibility;
```

Then update the Workshop/Library page/detail assemblers to consume the assessed compatibility data from the service layer and produce the new `compatibility` fields.

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell compatibility_assembly -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/models.rs apps/lwe/src-tauri/src/assembly/mod.rs apps/lwe/src-tauri/src/assembly/compatibility.rs apps/lwe/src-tauri/src/assembly/workshop_page.rs apps/lwe/src-tauri/src/assembly/workshop_detail.rs apps/lwe/src-tauri/src/assembly/library_page.rs apps/lwe/src-tauri/src/assembly/library_detail.rs
git commit -m "feat: assemble lwe compatibility summaries and explanations"
```

## Task 4: Render Compatibility Explanations in the Thin Frontend

**Files:**
- Create: `apps/lwe/src/lib/components/CompatibilityPanel.svelte`
- Modify: `apps/lwe/src/lib/types.ts`
- Modify: `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte`
- Modify: `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- Modify: `apps/lwe/src/lib/components/ItemCard.svelte`
- Modify: `apps/lwe/src/routes/workshop/+page.svelte`
- Modify: `apps/lwe/src/routes/library/+page.svelte`
- Test: `npm test --prefix apps/lwe`

- [ ] **Step 1: Write the failing frontend compatibility-panel test**

Create `apps/lwe/src/lib/components/CompatibilityPanel.svelte` with this Vitest/Svelte test first in a colocated test file `apps/lwe/src/lib/components/CompatibilityPanel.test.ts`:

```ts
import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import CompatibilityPanel from './CompatibilityPanel.svelte';

describe('CompatibilityPanel', () => {
  it('renders the explanation headline and detail', () => {
    render(CompatibilityPanel, {
      compatibility: {
        badge: 'unsupported',
        reasonCode: 'unsupported_web_item',
        headline: 'Web item not in first release',
        detail: 'Web Workshop items are recognized, but not yet supported.',
        nextStep: 'wait_for_future_support'
      }
    });

    expect(screen.getByText('Web item not in first release')).toBeTruthy();
    expect(screen.getByText('Web Workshop items are recognized, but not yet supported.')).toBeTruthy();
  });
});
```

- [ ] **Step 2: Run the frontend test to verify it fails**

Run: `npm test --prefix apps/lwe`
Expected: FAIL because the new compatibility panel and matching TS types do not exist yet.

- [ ] **Step 3: Mirror the new compatibility models into TypeScript**

In `apps/lwe/src/lib/types.ts`, add:

```ts
export type CompatibilityNextStep =
  | 'none'
  | 'open_in_steam'
  | 'resync_workshop_item'
  | 'wait_for_future_support';

export interface CompatibilitySummaryModel {
  badge: CompatibilityBadge;
  reasonCode: string;
}

export interface CompatibilityExplanationModel {
  badge: CompatibilityBadge;
  reasonCode: string;
  headline: string;
  detail: string;
  nextStep: CompatibilityNextStep;
}
```

Then update:

- `WorkshopItemSummary.compatibility`
- `WorkshopItemDetail.compatibility`
- `LibraryItemSummary.compatibility`
- `LibraryItemDetail.compatibility`

to use those structured types.

- [ ] **Step 4: Render the compatibility explanation panel and summary badges**

Create `apps/lwe/src/lib/components/CompatibilityPanel.svelte` with:

```svelte
<script lang="ts">
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { CompatibilityExplanationModel } from '$lib/types';

  export let compatibility: CompatibilityExplanationModel;
</script>

<section class="compatibility-panel">
  <StatusBadge label={compatibility.badge} />
  <h3>{compatibility.headline}</h3>
  <p>{compatibility.detail}</p>
  <p class="reason-code">{compatibility.reasonCode}</p>
  <p class="next-step">{compatibility.nextStep}</p>
</section>
```

Update `WorkshopDetailPanel.svelte` and `LibraryDetailPanel.svelte` to replace ad hoc compatibility text rendering with:

```svelte
<CompatibilityPanel compatibility={detail.compatibility} />
```

Update `ItemCard.svelte` so it reads the summary badge from `item.compatibility.badge` (or equivalent prop) instead of a plain string.

Update the Workshop/Library pages only as needed to pass the richer structured compatibility payloads through to the components.

- [ ] **Step 5: Run tests and commit**

Run: `npm test --prefix apps/lwe`
Expected: PASS

Then:

```bash
git add apps/lwe/src/lib/types.ts apps/lwe/src/lib/components/CompatibilityPanel.svelte apps/lwe/src/lib/components/CompatibilityPanel.test.ts apps/lwe/src/lib/components/WorkshopDetailPanel.svelte apps/lwe/src/lib/components/LibraryDetailPanel.svelte apps/lwe/src/lib/components/ItemCard.svelte apps/lwe/src/routes/workshop/+page.svelte apps/lwe/src/routes/library/+page.svelte
git commit -m "feat: render lwe compatibility explanations in frontend"
```

## Task 5: Mark the Compatibility Track as Implemented in Product Planning

**Files:**
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertions over the roadmap

- [ ] **Step 1: Update the roadmap wording for the compatibility track**

Adjust `docs/product/roadmap.md` so the `compatibility-evaluation-and-reporting` planning track reflects that the structured compatibility pipeline now exists, for example:

```md
- `compatibility-evaluation-and-reporting`: structured compatibility levels, reasons, and next-step guidance are now part of the active LWE shell; follow-on work should deepen runtime-specific reporting rather than invent the first explanation path
```

- [ ] **Step 2: Verify roadmap wording and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'structured compatibility levels, reasons, and next-step guidance' in roadmap
print('roadmap compatibility track updated')
PY
```

Expected: prints `roadmap compatibility track updated`.

Then:

```bash
git add docs/product/roadmap.md
git commit -m "docs: update lwe compatibility roadmap track"
```

## Self-Review Checklist

- Spec coverage:
  - visible compatibility levels → Tasks 1, 2, 3
  - explanation for why an item has that status → Tasks 1, 3, 4
  - next-step guidance → Tasks 1, 3, 4
  - Workshop + Library surfaces both show compatibility meaningfully → Tasks 2, 3, 4
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Type consistency:
  - `CompatibilityAssessment`, `CompatibilitySummary`, `CompatibilityExplanation`, and `CompatibilityNextStep` are introduced before later tasks use them.

## Expected Output of This Plan

When this plan is complete, LWE will have:

- a structured compatibility evaluation pipeline in the Rust core
- service-owned compatibility assessment instead of scattered strings
- assembler-owned compatibility summaries and explanations
- frontend rendering that shows compatibility badges, reasoned explanations, and next-step guidance in both Workshop and Library details

## Follow-on Plans After This One

The next plans after this file should cover:

1. runtime-specific compatibility reporting for `video` and `scene`
2. stronger desktop-shell and Library playback/application flows built on the same explanation model
