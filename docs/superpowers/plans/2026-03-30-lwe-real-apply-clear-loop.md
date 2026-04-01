# LWE Real Apply/Clear Loop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make one real, locally verifiable `Library -> Apply to monitor -> Desktop reflects result -> Clear` loop work on the current machine so LWE is demonstrably usable rather than only structurally complete.

**Architecture:** This plan builds on the real monitor discovery, JSON persistence, and current thin-frontend shell already in place. The work should stay narrow: pick the most reliable currently supported content/apply path, connect apply and clear to the actual backend/runtime path, keep Desktop and Library synchronized through the existing snapshot/action-outcome flow, and add explicit local verification steps as part of completion.

**Tech Stack:** Rust workspace, Tauri, `lwe-engine`, `lwe-library`, `lwe-core`, Svelte, pnpm, current local `Wayland + niri` environment

---

## Scope Note

This plan is intentionally about proving one real workflow, not maximizing feature breadth.

It includes:

- selecting a real monitor target
- applying one real supported item from Library
- reflecting that state in Desktop
- clearing it again
- manually verifying the full loop on the local machine

It does **not** include:

- preview-only mode
- all-monitor apply
- advanced restore/policy work
- broad runtime redesign

## File Map

### Files to modify

- `apps/lwe/src-tauri/src/services/desktop_service.rs` - connect apply/clear operations to the actual runtime/backend path instead of placeholder semantics
- `apps/lwe/src-tauri/src/services/library_service.rs` - ensure Library assignment state reflects the real applied result path
- `apps/lwe/src-tauri/src/results/desktop_apply.rs` - refine the apply/clear result shape if needed for the real loop
- `apps/lwe/src-tauri/src/assembly/action_outcome.rs` - ensure action outcomes reflect the real apply/clear path cleanly
- `apps/lwe/src-tauri/src/assembly/desktop_page.rs` - ensure Desktop reflects the applied/cleared state in a user-visible way
- `apps/lwe/src-tauri/src/assembly/library_page.rs` - ensure Library quick status reflects the same assignment state
- `apps/lwe/src-tauri/src/assembly/library_detail.rs` - same for the selected item detail
- `apps/lwe/src-tauri/src/commands/library.rs` - expose the real apply command path through the current command layer
- `apps/lwe/src-tauri/src/commands/desktop.rs` - expose the real clear command path through the current command layer
- `apps/lwe/src/lib/ipc.ts` - keep IPC wrappers aligned if the Rust command return shape changes slightly
- `apps/lwe/src/routes/library/+page.svelte` - preserve race-safe apply UX while reflecting the now-real action result
- `apps/lwe/src/routes/desktop/+page.svelte` - preserve Desktop refresh/clear UX while reflecting real applied state
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte` - keep the apply control and assignment feedback aligned with the real action loop
- `apps/lwe/src/lib/components/DesktopMonitorCard.svelte` - keep the clear control and current-state rendering aligned with the real action loop
- `docs/product/roadmap.md` - update wording once a real locally verified apply/clear loop exists

### Files to create

- `docs/archive/manual-verification/lwe-real-apply-clear-loop.md` - record of the exact local verification path and observed outcome on the current machine

### Files to inspect while implementing

- `apps/lwe/src-tauri/src/services/desktop_service.rs`
- `apps/lwe/src-tauri/src/commands/library.rs`
- `apps/lwe/src-tauri/src/commands/desktop.rs`
- `apps/lwe/src/routes/library/+page.svelte`
- `apps/lwe/src/routes/desktop/+page.svelte`
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- `apps/lwe/src/lib/components/DesktopMonitorCard.svelte`

## Task 1: Identify and Lock a Real Apply Path for the Current Environment

**Files:**
- Modify: `apps/lwe/src-tauri/src/services/desktop_service.rs`
- Test: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`

- [ ] **Step 1: Add a failing test for the real apply path contract**

Add a new test to `apps/lwe/src-tauri/src/services/desktop_service.rs` that asserts `apply_to_monitor` no longer returns an immediate “unavailable” placeholder result when given a monitor discovered by the current backend and a real supported Library item fixture.

The test should be minimal but concrete, for example by asserting that the returned application result is not the same placeholder/degraded outcome used before.

- [ ] **Step 2: Run the test to confirm the current placeholder path fails**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: the new test fails because the apply path is not yet really connected.

- [ ] **Step 3: Connect `DesktopService::apply_to_monitor` to the real backend path**

Replace the placeholder success/failure path with the smallest real path that can work on the current machine.

Requirements for this step:

- use the real monitor discovery result
- use one real currently supported item path from Library/workshop-projected content
- call through the actual backend/runtime path instead of only simulating success
- keep failure explicit if the environment cannot execute the operation

Do **not** broaden the implementation to every future runtime scenario. This is about one real path.

- [ ] **Step 4: Re-run the desktop apply test**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: the new apply-path test passes, and the surrounding desktop-flow tests still pass.

- [ ] **Step 5: Commit**

```bash
git add apps/lwe/src-tauri/src/services/desktop_service.rs
git commit -m "feat: connect desktop apply to real backend path"
```

## Task 2: Make Clear Use the Same Real State Path

**Files:**
- Modify: `apps/lwe/src-tauri/src/services/desktop_service.rs`
- Modify: `apps/lwe/src-tauri/src/results/desktop_apply.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/action_outcome.rs`
- Test: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`

- [ ] **Step 1: Add a failing clear-path test**

Add a test that proves `clear_monitor` acts on the same real state path as apply rather than merely clearing persisted metadata while leaving the current desktop state untouched.

- [ ] **Step 2: Run the test to confirm the current behavior fails**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: the new clear-path test fails if the current implementation is still too placeholder-like.

- [ ] **Step 3: Implement real clear semantics**

Update the Desktop service and any directly related result/action-outcome code so clear acts on the same real desktop state path used by apply.

Constraints:

- keep it narrow to the current supported environment/path
- keep failure explicit when the backend cannot clear
- do not introduce unrelated Desktop controls

- [ ] **Step 4: Re-run the desktop apply/clear tests**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/lwe/src-tauri/src/services/desktop_service.rs apps/lwe/src-tauri/src/results/desktop_apply.rs apps/lwe/src-tauri/src/assembly/action_outcome.rs
git commit -m "feat: connect desktop clear to real backend path"
```

## Task 3: Keep Library and Desktop in Sync With the Real Action Results

**Files:**
- Modify: `apps/lwe/src-tauri/src/services/library_service.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/desktop_page.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/library_page.rs`
- Modify: `apps/lwe/src-tauri/src/assembly/library_detail.rs`
- Modify: `apps/lwe/src-tauri/src/commands/library.rs`
- Modify: `apps/lwe/src-tauri/src/commands/desktop.rs`
- Test: `cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture`

- [ ] **Step 1: Add a failing coherence test**

Add a test that proves the same real applied state is visible in both:

- the Desktop snapshot
- the current Library quick-status/detail path

and that clear removes that state coherently from both.

- [ ] **Step 2: Run the test to confirm the current state still drifts**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: the new coherence test fails before the state propagation is corrected.

- [ ] **Step 3: Unify the active state propagation**

Update the services/assemblers/commands listed above so Library and Desktop read from one coherent real assignment state after apply/clear.

Do not add new frontend flows here; this is backend-to-snapshot coherence work.

- [ ] **Step 4: Re-run tests**

Run:

```bash
cargo test -p lwe-app-shell desktop_apply_flow -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/lwe/src-tauri/src/services/library_service.rs apps/lwe/src-tauri/src/assembly/desktop_page.rs apps/lwe/src-tauri/src/assembly/library_page.rs apps/lwe/src-tauri/src/assembly/library_detail.rs apps/lwe/src-tauri/src/commands/library.rs apps/lwe/src-tauri/src/commands/desktop.rs
git commit -m "fix: keep library and desktop state in sync"
```

## Task 4: Verify the Real Loop Through the Existing Frontend

**Files:**
- Modify: `apps/lwe/src/lib/ipc.ts` only if required to align with the final action contract
- Modify: `apps/lwe/src/routes/library/+page.svelte` only if required to surface the real apply result cleanly
- Modify: `apps/lwe/src/routes/desktop/+page.svelte` only if required to surface the real clear result cleanly
- Modify: `apps/lwe/src/lib/components/LibraryDetailPanel.svelte` only if required to keep assignment feedback aligned
- Modify: `apps/lwe/src/lib/components/DesktopMonitorCard.svelte` only if required to keep clear feedback aligned
- Create: `docs/archive/manual-verification/lwe-real-apply-clear-loop.md`
- Test: `pnpm --dir apps/lwe test && pnpm --dir apps/lwe check`

- [ ] **Step 1: Add the failing integration assertion (if needed)**

If the frontend contract still needs a small final alignment for the real action path, add the smallest failing test to capture it.

If no frontend contract change is needed, skip directly to the manual verification path.

- [ ] **Step 2: Run the frontend verification path**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

- [ ] **Step 3: Perform manual local verification**

On the current machine, verify the loop end-to-end:

1. launch LWE
2. open Library
3. select one compatible item
4. choose one real monitor
5. apply it
6. confirm Desktop reflects the result
7. clear it from Desktop
8. confirm Desktop state updates again

Record what actually happened in:

- `docs/archive/manual-verification/lwe-real-apply-clear-loop.md`

Include:

- date/time
- environment summary (`Wayland + niri`)
- which item/monitor path was tested
- whether apply succeeded
- whether Desktop reflected it
- whether clear succeeded
- any residual limitations

- [ ] **Step 4: Commit**

```bash
git add apps/lwe/src/lib/ipc.ts apps/lwe/src/routes/library/+page.svelte apps/lwe/src/routes/desktop/+page.svelte apps/lwe/src/lib/components/LibraryDetailPanel.svelte apps/lwe/src/lib/components/DesktopMonitorCard.svelte docs/archive/manual-verification/lwe-real-apply-clear-loop.md
git commit -m "feat: verify real apply clear loop on local desktop"
```

## Task 5: Update the Roadmap to Reflect a Real Verified Loop

**Files:**
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertion over the roadmap

- [ ] **Step 1: Update the roadmap wording**

Adjust the `desktop-shell-and-library-flow` wording so it reflects the now-verified state. For example, once the local verification succeeds:

```md
- `desktop-shell-and-library-flow`: the active LWE shell now supports a locally verified `Library -> Apply to monitor -> Desktop reflects result -> Clear` loop on the current Wayland + niri path; follow-on work should broaden runtime coverage and interaction polish rather than establish the first real desktop action path
```

- [ ] **Step 2: Verify roadmap wording and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'Library -> Apply to monitor -> Desktop reflects result -> Clear' in roadmap
print('real apply clear roadmap wording updated')
PY
```

Expected: prints `real apply clear roadmap wording updated`.

Then:

```bash
git add docs/product/roadmap.md
git commit -m "docs: update roadmap for real apply clear loop"
```

## Self-Review Checklist

- Spec coverage:
  - one real apply path → Task 1
  - one real clear path → Task 2
  - Library/Desktop coherence → Task 3
  - local manual verification → Task 4
  - roadmap updated to the new reality → Task 5
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Scope check:
  - the work remains focused on one real loop rather than broad runtime support
  - local machine verification is part of completion, not an optional afterthought

## Expected Output of This Plan

When this plan is complete, LWE will have one real, locally verified desktop action loop on the current machine. That will make the product materially more credible than any amount of additional shell or architecture work without real action verification.
