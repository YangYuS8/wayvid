# LWE Usability Pass v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the current LWE shell easier to understand and use by clarifying `Library` vs `Workshop`, making the apply action obvious and fast, and tightening the detail layout so it supports real use instead of wasting space.

**Architecture:** This is a product-clarity pass, not a feature-platform refactor. The data flow and shell foundations remain intact; the work concentrates on page semantics, action discoverability, and layout density. The implementation should preserve the existing thin-frontend model while making the current capabilities easier to understand and actually use.

**Tech Stack:** Svelte 5, Tauri 2, TypeScript, pnpm, Tailwind/shadcn-based UI foundation, existing Rust/Tauri command layer

---

## Scope Note

This pass intentionally focuses on three usability problems only:

1. `Library` vs `Workshop` role confusion
2. invisible/awkward `Apply` entry points
3. oversized detail-panel layout

It does **not** include:

- deeper Settings work
- true online Workshop browsing
- new runtime capability work
- broad visual redesign beyond the already-established UI foundation

## File Map

### Files to create

- `src/lib/components/ItemActionsMenu.svelte` - right-click / secondary action menu for wallpaper items

### Files to modify

- `src/routes/library/+page.svelte` - clarify Library framing and wire stronger apply discoverability
- `src/routes/workshop/+page.svelte` - clarify current local-sync Workshop role in page framing/copy
- `src/lib/components/ItemCard.svelte` - support the visible apply affordance and context-menu affordance
- `src/lib/components/LibraryDetailPanel.svelte` - move to a denser vertical information flow with visible primary apply action
- `src/lib/components/WorkshopDetailPanel.svelte` - same vertical-density improvements and role clarity
- `src/lib/components/MonitorPicker.svelte` - align with the stronger visible-apply path if needed
- `src/lib/types.ts` - only if a minimal UI-facing action/secondary-action prop shape is needed
- `docs/product/roadmap.md` - reflect the new usability pass if completed

### Files to inspect while implementing

- `src/routes/library/+page.svelte`
- `src/routes/workshop/+page.svelte`
- `src/lib/components/ItemCard.svelte`
- `src/lib/components/LibraryDetailPanel.svelte`
- `src/lib/components/WorkshopDetailPanel.svelte`
- `src/lib/components/MonitorPicker.svelte`
- `docs/superpowers/specs/2026-03-31-lwe-usability-pass-v1-design.md`

## Task 1: Clarify `Library` vs `Workshop` Page Semantics

**Files:**
- Modify: `src/routes/library/+page.svelte`
- Modify: `src/routes/workshop/+page.svelte`
- Test: `pnpm --dir  test`

- [ ] **Step 1: Add a failing page-semantics test**

Add or update render tests so they assert:

- `Library` is framed as the local/owned content surface
- `Workshop` is framed as the current local-sync Workshop view, not the full online browser

This can be done through existing page render tests or new lightweight text assertions.

- [ ] **Step 2: Run the frontend tests to verify failure**

Run:

```bash
pnpm --dir  test
```

Expected: FAIL once the new copy/semantics assertions are introduced.

- [ ] **Step 3: Update page framing and copy**

Update the page headers/subtitles and any local explanatory copy so the distinction becomes explicit:

- `Library` = already-owned / locally available content
- `Workshop` = currently a view over locally synchronized Workshop content

Do not rename the page entirely, and do not fake online Workshop browsing.

- [ ] **Step 4: Run verification**

Run:

```bash
pnpm --dir  test && pnpm --dir  check
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/routes/library/+page.svelte src/routes/workshop/+page.svelte
git commit -m "feat: clarify library and workshop roles"
```

## Task 2: Make Apply Obvious and Fast

**Files:**
- Create: `src/lib/components/ItemActionsMenu.svelte`
- Modify: `src/lib/components/ItemCard.svelte`
- Modify: `src/lib/components/LibraryDetailPanel.svelte`
- Modify: `src/routes/library/+page.svelte`
- Test: `pnpm --dir  test`

- [ ] **Step 1: Add a failing apply-discoverability test**

Add or update tests so they assert:

- a visible primary `Apply` action exists in the Library detail path
- a secondary/context action path exists on item surfaces

The test does not need to validate runtime behavior, only UI discoverability and presence.

- [ ] **Step 2: Run the frontend tests to verify failure**

Run:

```bash
pnpm --dir  test
```

Expected: FAIL once the new discoverability assertions are in place.

- [ ] **Step 3: Implement the visible primary action and secondary menu**

Add a small `ItemActionsMenu` and update the Library item/detail surfaces so:

- the detail panel exposes the clear visible primary `Apply` action
- item surfaces expose a secondary context/quick-action path
- the current monitor-picker flow is reused instead of reworked

Do not move ownership of the apply business logic out of the current detail/panel path; this is a discoverability improvement, not a flow rewrite.

- [ ] **Step 4: Run verification**

Run:

```bash
pnpm --dir  test && pnpm --dir  check
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/ItemActionsMenu.svelte src/lib/components/ItemCard.svelte src/lib/components/LibraryDetailPanel.svelte src/routes/library/+page.svelte
git commit -m "feat: improve apply action discoverability"
```

## Task 3: Tighten the Detail Panel Layout

**Files:**
- Modify: `src/lib/components/LibraryDetailPanel.svelte`
- Modify: `src/lib/components/WorkshopDetailPanel.svelte`
- Modify: `src/lib/components/CoverImage.svelte` only if needed for the new compact detail treatment
- Test: `pnpm --dir  test`

- [ ] **Step 1: Add a failing layout-density test**

Add or update tests so they assert the detail panels now follow the denser vertical structure rather than a large two-column media layout.

This can be done by checking for the new section ordering / wrappers / structure rather than pixel-perfect output.

- [ ] **Step 2: Run the frontend tests to verify failure**

Run:

```bash
pnpm --dir  test
```

Expected: FAIL once the structural expectations are added.

- [ ] **Step 3: Rework the detail panels into a compact vertical flow**

Update `LibraryDetailPanel` and `WorkshopDetailPanel` so they follow this order:

1. header
2. quick status
3. actions
4. compact cover
5. compatibility
6. metadata

Requirements:

- reduce the dominance of cover art
- eliminate the low-density “cover column + text column” feel
- ensure long text still wraps safely
- preserve existing data and actions

- [ ] **Step 4: Run verification**

Run:

```bash
pnpm --dir  test && pnpm --dir  check
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/LibraryDetailPanel.svelte src/lib/components/WorkshopDetailPanel.svelte src/lib/components/CoverImage.svelte
git commit -m "feat: tighten detail panel layout for real use"
```

## Task 4: Align Roadmap Wording With the Usability Pass

**Files:**
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertion over the roadmap

- [ ] **Step 1: Update roadmap wording**

Adjust the roadmap so the next iteration explicitly reflects that the usability pass clarified page semantics, made apply actions more discoverable, and tightened the detail layout.

Example target wording:

```md
- `lwe-usability-pass-v1`: the active shell now distinguishes local Library content from the current synced-Workshop view more clearly, exposes a stronger Apply entry path, and uses denser detail panels that better support real use; follow-on work can deepen Settings and online Workshop browsing rather than solving first-use confusion
```

- [ ] **Step 2: Verify roadmap wording and commit**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'stronger Apply entry path' in roadmap
print('lwe usability roadmap wording updated')
PY
```

Expected: prints `lwe usability roadmap wording updated`.

Then:

```bash
git add docs/product/roadmap.md
git commit -m "docs: update lwe usability roadmap track"
```

## Self-Review Checklist

- Spec coverage:
  - page-role clarification → Task 1
  - apply discoverability → Task 2
  - detail density/layout correction → Task 3
  - roadmap update → Task 4
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Scope check:
  - no Settings expansion
  - no true online Workshop work
  - no backend/runtime capability expansion

## Expected Output of This Plan

When this plan is complete, LWE should be much easier to understand and use on first contact:

- `Library` and `Workshop` will no longer feel like the same page
- `Apply` will be visible and easier to discover
- detail panels will feel action-oriented instead of oversized and wasteful
