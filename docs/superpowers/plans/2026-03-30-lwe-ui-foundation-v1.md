# LWE UI Foundation v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce `shadcn-svelte` as the UI primitive layer for LWE and upgrade the shell plus core product components so `Library`, `Workshop`, and `Desktop` read as one coherent desktop application.

**Architecture:** This is a medium UI refresh, not a total redesign. The implementation should first establish the shared shell and primitive layer, then rebuild the most visible product components (`ItemCard`, detail panels, desktop monitor cards, state badges) on top of that foundation. Backend/domain logic stays unchanged except for the minimum data plumbing needed to support the new shell/actions layout.

**Tech Stack:** Svelte 5, Tauri 2, TypeScript, pnpm, `shadcn-svelte`, Tailwind-compatible utility styling or equivalent local styles, Vitest

---

## Scope Note

This plan intentionally focuses on **UI foundation + core product components**.

It includes:

- `shadcn-svelte` setup
- shell/navigation redesign
- shared primitive integration
- medium refresh of Library / Workshop / Desktop components
- feedback primitives where needed to support the new shell

It does **not** include:

- complete redesign of every page detail
- broad animation systems
- large backend/domain refactors
- dark-mode/theme-system expansion beyond what the UI base needs

## File Map

### Files to create

- `apps/lwe/components.json` - `shadcn-svelte` component registry/config for the app
- `apps/lwe/src/lib/ui/**` - imported/generated `shadcn-svelte` primitive components used by the shell and page surfaces
- `apps/lwe/src/lib/layout/AppShell.svelte` - the new persistent desktop shell wrapper
- `apps/lwe/src/lib/layout/PageHeader.svelte` - shared page header block for title/subtitle/actions
- `apps/lwe/src/lib/theme/tokens.ts` - small shared UI token map if needed for app-level constants
- `apps/lwe/src/lib/components/DesktopMonitorCard.svelte` - proper product-level monitor card component replacing inline Desktop card markup

### Files to modify

- `apps/lwe/package.json` - add the UI dependencies needed for `shadcn-svelte`
- `apps/lwe/src/routes/+layout.svelte` - replace the current minimal nav shell with the new `AppShell`
- `apps/lwe/src/routes/library/+page.svelte` - align page structure with the new shell and page header pattern
- `apps/lwe/src/routes/workshop/+page.svelte` - same
- `apps/lwe/src/routes/desktop/+page.svelte` - same
- `apps/lwe/src/routes/settings/+page.svelte` - adopt the shell and basic page chrome even if deep redesign is deferred
- `apps/lwe/src/lib/components/ItemCard.svelte` - medium visual/system rewrite against the new primitive layer
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte` - medium visual/system rewrite
- `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte` - medium visual/system rewrite
- `apps/lwe/src/lib/components/CompatibilityPanel.svelte` - align with the new status/info panel styling
- `apps/lwe/src/lib/components/StatusBadge.svelte` - align with the primitive layer or wrap `shadcn-svelte` badge primitives
- `apps/lwe/src/lib/components/CoverImage.svelte` - align with the new card/media language
- `apps/lwe/src/routes/library/+page.svelte` / `workshop/+page.svelte` / `desktop/+page.svelte` tests - adjust snapshots/assertions if needed for the new shell/components

### Files to inspect while implementing

- `apps/lwe/src/routes/+layout.svelte`
- `apps/lwe/src/routes/library/+page.svelte`
- `apps/lwe/src/routes/workshop/+page.svelte`
- `apps/lwe/src/routes/desktop/+page.svelte`
- `apps/lwe/src/lib/components/ItemCard.svelte`
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte`
- `apps/lwe/src/lib/components/CompatibilityPanel.svelte`
- `docs/superpowers/specs/2026-03-30-lwe-ui-foundation-v1-design.md`

## Task 1: Add `shadcn-svelte` and Primitive UI Foundation

**Files:**
- Create: `apps/lwe/components.json`
- Create: `apps/lwe/src/lib/ui/**`
- Modify: `apps/lwe/package.json`
- Test: `pnpm --dir apps/lwe check`

- [ ] **Step 1: Write the failing dependency/config check**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
pkg = Path('apps/lwe/package.json').read_text()
assert 'shadcn-svelte' in pkg or '@shadcn-svelte' in pkg
assert Path('apps/lwe/components.json').exists()
print('ui foundation config present')
PY
```

Expected: FAIL because the current frontend does not yet include `shadcn-svelte` or its config.

- [ ] **Step 2: Add the UI dependencies and config**

Update `apps/lwe/package.json` to add the minimal UI dependencies needed for `shadcn-svelte` primitives used in this phase.

Create `apps/lwe/components.json` with the minimal configuration that points generated/imported primitives into `src/lib/ui`.

Create/import only the primitive components actually needed in this phase, prioritizing:

- button
- card
- badge
- select
- dialog
- separator

Do not import the entire catalog.

- [ ] **Step 3: Add a tiny primitive wrapper smoke test**

Create a small smoke test for one primitive under `apps/lwe/src/lib/ui/**` (or a wrapper around it) so the foundation is exercised by the frontend test runner.

- [ ] **Step 4: Run verification**

Run:

```bash
pnpm --dir apps/lwe install && pnpm --dir apps/lwe check
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/lwe/package.json apps/lwe/components.json apps/lwe/src/lib/ui
git commit -m "feat: add lwe shadcn ui foundation"
```

## Task 2: Rebuild the Global LWE Shell

**Files:**
- Create: `apps/lwe/src/lib/layout/AppShell.svelte`
- Create: `apps/lwe/src/lib/layout/PageHeader.svelte`
- Modify: `apps/lwe/src/routes/+layout.svelte`
- Test: `pnpm --dir apps/lwe test`

- [ ] **Step 1: Write the failing shell-layout test**

Add a frontend test that expects the new layout shell to include:

- persistent navigation
- current-page highlighting affordance
- a main content region

This can be a simple render-level test of `AppShell.svelte` or `+layout.svelte`.

- [ ] **Step 2: Run the frontend test to verify it fails**

Run:

```bash
pnpm --dir apps/lwe test
```

Expected: FAIL because the current shell is still just a flat link list.

- [ ] **Step 3: Implement `AppShell` and `PageHeader`**

Create `AppShell.svelte` to provide:

- persistent navigation for `Library / Workshop / Desktop / Settings`
- current-page emphasis
- main content wrapper

Create `PageHeader.svelte` to provide:

- eyebrow/title/subtitle layout
- optional action slot

Then update `+layout.svelte` to use the new shell.

- [ ] **Step 4: Run verification**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/lwe/src/lib/layout apps/lwe/src/routes/+layout.svelte
git commit -m "feat: add lwe application shell"
```

## Task 3: Rebuild Core Product Components on the New Primitive Layer

**Files:**
- Create: `apps/lwe/src/lib/components/DesktopMonitorCard.svelte`
- Modify: `apps/lwe/src/lib/components/ItemCard.svelte`
- Modify: `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- Modify: `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte`
- Modify: `apps/lwe/src/lib/components/CompatibilityPanel.svelte`
- Modify: `apps/lwe/src/lib/components/StatusBadge.svelte`
- Modify: `apps/lwe/src/lib/components/CoverImage.svelte`
- Test: `pnpm --dir apps/lwe test`

- [ ] **Step 1: Write the failing component-system test**

Add one or more tests that expect:

- `ItemCard` to render through the new card/badge primitive structure
- `DesktopMonitorCard` to exist and render monitor state
- detail panels to render within the new component language

- [ ] **Step 2: Run the frontend tests to verify failure**

Run:

```bash
pnpm --dir apps/lwe test
```

Expected: FAIL because the old components are still manually styled and `DesktopMonitorCard` does not exist.

- [ ] **Step 3: Rebuild the components**

Implement `DesktopMonitorCard.svelte` and rebuild the existing components so they share one visual grammar:

- media-first card treatment
- stronger status hierarchy
- cleaner detail panel structure
- unified spacing, border, badge, and surface treatment

Keep the business props and data flow intact. This is a UI refoundation task, not a domain rewrite.

- [ ] **Step 4: Run verification**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/lwe/src/lib/components
git commit -m "feat: refresh lwe core product components"
```

## Task 4: Integrate the New Shell and Components Into Library / Workshop / Desktop Pages

**Files:**
- Modify: `apps/lwe/src/routes/library/+page.svelte`
- Modify: `apps/lwe/src/routes/workshop/+page.svelte`
- Modify: `apps/lwe/src/routes/desktop/+page.svelte`
- Modify: `apps/lwe/src/routes/settings/+page.svelte`
- Test: `pnpm --dir apps/lwe test`

- [ ] **Step 1: Write the failing page-integration test**

Add tests or update existing page render tests so they expect:

- page headers to use the new shell/header structure
- Desktop to render through `DesktopMonitorCard`
- Library / Workshop to sit inside the new shell/page chrome

- [ ] **Step 2: Run the frontend tests to verify failure**

Run:

```bash
pnpm --dir apps/lwe test
```

Expected: FAIL because the current pages still use the old page shell and inline structures.

- [ ] **Step 3: Integrate the new shell/components**

Update:

- `Library` page to use `PageHeader` and the refreshed `ItemCard` / `LibraryDetailPanel`
- `Workshop` page to use `PageHeader` and the refreshed `ItemCard` / `WorkshopDetailPanel`
- `Desktop` page to render `DesktopMonitorCard` within the new shell
- `Settings` page to adopt the shell and base page chrome even if it stays otherwise simple

Do not redesign the page information architecture.

- [ ] **Step 4: Run verification**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/lwe/src/routes
git commit -m "feat: integrate lwe ui foundation into core pages"
```

## Task 5: Add Interaction Feedback Primitives Where the New Shell Needs Them

**Files:**
- Modify: active page/component files as needed for dialog/select/toast integration
- Test: `pnpm --dir apps/lwe test`

- [ ] **Step 1: Identify the minimum feedback surfaces to standardize**

Limit this task to the places where the new shell/components clearly benefit from the primitive layer, such as:

- monitor selection UI
- local action feedback in Library / Workshop / Desktop
- consistent button/select affordances

- [ ] **Step 2: Implement the minimal feedback upgrades**

Replace ad hoc controls/feedback surfaces with the already-adopted primitive layer where that meaningfully improves consistency.

Do not turn this into a broad interaction redesign.

- [ ] **Step 3: Run verification**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add apps/lwe/src/lib/components apps/lwe/src/routes
git commit -m "feat: polish lwe interaction feedback surfaces"
```

## Self-Review Checklist

- Spec coverage:
  - App shell/navigation/page frame → Task 2
  - core component appearance refresh → Task 3
  - Library / Workshop / Desktop integration → Task 4
  - interaction feedback primitives → Task 5
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Scope check:
  - no broad backend/domain work added
  - no full visual reinvention attempted
  - `shadcn-svelte` stays a primitive layer, not a page-template system

## Expected Output of This Plan

When this plan is complete, LWE should feel like one coherent desktop product with:

- a real application shell
- unified page chrome
- refreshed core components
- a stable primitive component foundation
- a much clearer path for future UI work without ad hoc styling drift
