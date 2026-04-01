# LWE UI Foundation v2 Tailwind Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce Tailwind CSS and migrate the already-established LWE UI foundation surfaces onto a unified Tailwind + `shadcn-svelte` styling system without expanding into a whole-app redesign.

**Architecture:** This is a style-system consolidation pass. It keeps the current shell structure, component boundaries, and thin-frontend contracts, but replaces the mixed local-`<style>` approach with a coherent Tailwind-based primitive + component layer. The migration targets only the already-approved UI foundation surfaces: shell, page headers, and core product components for Library / Workshop / Desktop / Settings.

**Tech Stack:** Svelte 5, Tauri 2, TypeScript, pnpm, Tailwind CSS, `shadcn-svelte`, Bits UI, Vitest

---

## Scope Note

This plan is intentionally limited to the **UI foundation v1 surfaces**.

It includes:

- Tailwind setup in `apps/lwe`
- aligning the primitive layer with Tailwind usage
- migrating shell + core product components to the unified styling system
- updating page integration only as needed to consume the migrated components

It does **not** include:

- converting every frontend file to Tailwind
- broad redesign of unrelated page internals
- domain/backend refactors
- advanced animation/theming work

## File Map

### Files to create

- `apps/lwe/src/app.css` - Tailwind entrypoint and the minimal app-level tokens/layers for the foundation
- `apps/lwe/tailwind.config.ts` - Tailwind configuration for the LWE shell and primitive layer
- `apps/lwe/postcss.config.cjs` - Tailwind/PostCSS integration

### Files to modify

- `apps/lwe/package.json` - add Tailwind-related dependencies and scripts if needed
- `apps/lwe/components.json` - align the primitive config with Tailwind usage
- `apps/lwe/src/routes/+layout.svelte` - wire in the global app stylesheet and shell class usage
- `apps/lwe/src/lib/layout/AppShell.svelte`
- `apps/lwe/src/lib/layout/PageHeader.svelte`
- `apps/lwe/src/lib/ui/button/button.svelte`
- `apps/lwe/src/lib/ui/badge/badge.svelte`
- `apps/lwe/src/lib/ui/card/card.svelte`
- `apps/lwe/src/lib/ui/dialog/*.svelte`
- `apps/lwe/src/lib/ui/select/*.svelte`
- `apps/lwe/src/lib/ui/separator/separator.svelte`
- `apps/lwe/src/lib/components/ItemCard.svelte`
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte`
- `apps/lwe/src/lib/components/DesktopMonitorCard.svelte`
- `apps/lwe/src/lib/components/CompatibilityPanel.svelte`
- `apps/lwe/src/lib/components/StatusBadge.svelte`
- `apps/lwe/src/lib/components/CoverImage.svelte`
- `apps/lwe/src/routes/library/+page.svelte`
- `apps/lwe/src/routes/workshop/+page.svelte`
- `apps/lwe/src/routes/desktop/+page.svelte`
- `apps/lwe/src/routes/settings/+page.svelte`

### Files to inspect while implementing

- `apps/lwe/src/routes/+layout.svelte`
- `apps/lwe/src/lib/layout/AppShell.svelte`
- `apps/lwe/src/lib/components/ItemCard.svelte`
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte`
- `apps/lwe/src/lib/components/DesktopMonitorCard.svelte`
- `docs/superpowers/specs/2026-03-30-lwe-ui-foundation-v2-tailwind-design.md`

## Task 1: Add Tailwind to the LWE Frontend

**Files:**
- Create: `apps/lwe/src/app.css`
- Create: `apps/lwe/tailwind.config.ts`
- Create: `apps/lwe/postcss.config.cjs`
- Modify: `apps/lwe/package.json`
- Modify: `apps/lwe/components.json`
- Test: `pnpm --dir apps/lwe check`

- [ ] **Step 1: Write the failing Tailwind setup check**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
assert Path('apps/lwe/src/app.css').exists()
assert Path('apps/lwe/tailwind.config.ts').exists()
assert Path('apps/lwe/postcss.config.cjs').exists()
print('tailwind config present')
PY
```

Expected: FAIL because the Tailwind setup files do not exist yet.

- [ ] **Step 2: Add Tailwind dependencies and configuration**

Update `apps/lwe/package.json` with the minimal Tailwind dependencies needed for Svelte + `shadcn-svelte` integration.

Create `apps/lwe/tailwind.config.ts` with a content glob that covers:

- `src/**/*.{svelte,ts}`
- `src/lib/ui/**/*.{svelte,ts}`

Create `apps/lwe/postcss.config.cjs` to enable Tailwind and autoprefixer.

Update `apps/lwe/components.json` so the primitive layer aligns with the Tailwind setup.

- [ ] **Step 3: Create the app stylesheet**

Create `apps/lwe/src/app.css` with the minimal Tailwind layers and any token variables the current shell/components need in this phase.

- [ ] **Step 4: Wire the app stylesheet into the shell**

Update `apps/lwe/src/routes/+layout.svelte` to import the new global stylesheet.

- [ ] **Step 5: Run verification and commit**

Run:

```bash
pnpm --dir apps/lwe install && pnpm --dir apps/lwe check
```

Expected: PASS

Then:

```bash
git add apps/lwe/package.json apps/lwe/pnpm-lock.yaml apps/lwe/components.json apps/lwe/tailwind.config.ts apps/lwe/postcss.config.cjs apps/lwe/src/app.css apps/lwe/src/routes/+layout.svelte
git commit -m "feat: add tailwind foundation to lwe"
```

## Task 2: Align the Primitive Layer With Tailwind

**Files:**
- Modify: `apps/lwe/src/lib/ui/button/button.svelte`
- Modify: `apps/lwe/src/lib/ui/badge/badge.svelte`
- Modify: `apps/lwe/src/lib/ui/card/card.svelte`
- Modify: `apps/lwe/src/lib/ui/dialog/*.svelte`
- Modify: `apps/lwe/src/lib/ui/select/*.svelte`
- Modify: `apps/lwe/src/lib/ui/separator/separator.svelte`
- Test: `pnpm --dir apps/lwe test`

- [ ] **Step 1: Write the failing primitive-style test**

Add or update a primitive test so it asserts one of the current primitives is now rendered through the Tailwind-based style path rather than the older ad hoc class/style mix.

- [ ] **Step 2: Run the frontend tests to verify the current primitive styling is not yet aligned**

Run:

```bash
pnpm --dir apps/lwe test
```

Expected: FAIL once the new assertions are in place.

- [ ] **Step 3: Migrate the primitives to the Tailwind-based system**

Update the existing primitive files so they use a coherent Tailwind class composition strategy:

- utility-based spacing/surfaces/borders
- the existing `cn(...)` helper path
- `class-variance-authority` where it is already justified

Do not expand the primitive catalog; just align the current one.

- [ ] **Step 4: Run verification and commit**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

Then:

```bash
git add apps/lwe/src/lib/ui
git commit -m "refactor: align lwe primitives with tailwind foundation"
```

## Task 3: Migrate the Shell and Core Product Components

**Files:**
- Modify: `apps/lwe/src/lib/layout/AppShell.svelte`
- Modify: `apps/lwe/src/lib/layout/PageHeader.svelte`
- Modify: `apps/lwe/src/lib/components/ItemCard.svelte`
- Modify: `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- Modify: `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte`
- Modify: `apps/lwe/src/lib/components/DesktopMonitorCard.svelte`
- Modify: `apps/lwe/src/lib/components/CompatibilityPanel.svelte`
- Modify: `apps/lwe/src/lib/components/StatusBadge.svelte`
- Modify: `apps/lwe/src/lib/components/CoverImage.svelte`
- Test: `pnpm --dir apps/lwe test`

- [ ] **Step 1: Write the failing component-foundation test**

Add or update component tests so they assert the new shell/components consume the Tailwind/styled primitive layer rather than their old local `<style>` grammar.

- [ ] **Step 2: Run tests to verify the current components are still using the mixed style system**

Run:

```bash
pnpm --dir apps/lwe test
```

Expected: FAIL once the new assertions are in place.

- [ ] **Step 3: Migrate the components to the unified style system**

Update the shell and component files so they:

- rely on Tailwind/primitive classes rather than page-local style blocks where practical
- keep the same data flow and product semantics
- preserve the approved media-product direction

Do not redesign the component APIs.

- [ ] **Step 4: Run verification and commit**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

Then:

```bash
git add apps/lwe/src/lib/layout apps/lwe/src/lib/components
git commit -m "refactor: migrate lwe shell and components to tailwind"
```

## Task 4: Re-integrate the Migrated Components Into the Core Pages

**Files:**
- Modify: `apps/lwe/src/routes/library/+page.svelte`
- Modify: `apps/lwe/src/routes/workshop/+page.svelte`
- Modify: `apps/lwe/src/routes/desktop/+page.svelte`
- Modify: `apps/lwe/src/routes/settings/+page.svelte`
- Test: `pnpm --dir apps/lwe test`

- [ ] **Step 1: Write the failing page-integration check**

Add or update page render tests so they assert the pages are using the migrated shell/components without regressing current state/interaction rendering.

- [ ] **Step 2: Run tests to verify integration still needs alignment**

Run:

```bash
pnpm --dir apps/lwe test
```

Expected: FAIL once the new assertions are in place.

- [ ] **Step 3: Update the page files**

Adjust the route files only as needed so the migrated shell/components render correctly in:

- `Library`
- `Workshop`
- `Desktop`
- `Settings`

Do not redesign page data flow.

- [ ] **Step 4: Run verification and commit**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

Then:

```bash
git add apps/lwe/src/routes
git commit -m "feat: integrate tailwind ui foundation into core pages"
```

## Task 5: Polish the Minimal Feedback Surfaces Under the Unified System

**Files:**
- Modify: active component/page files as needed for the current monitor-selection / status / degraded-state feedback surfaces
- Test: `pnpm --dir apps/lwe test`

- [ ] **Step 1: Identify the smallest necessary feedback polish**

Limit this to the places already touched by the UI foundation where Tailwind + primitive alignment clearly benefits feedback/readability.

- [ ] **Step 2: Apply the minimal polish pass**

Adjust only the current feedback surfaces that are already in the UI foundation scope so they fit the unified styling system.

Avoid broad redesign of interaction flows.

- [ ] **Step 3: Run verification and commit**

Run:

```bash
pnpm --dir apps/lwe test && pnpm --dir apps/lwe check
```

Expected: PASS

Then:

```bash
git add apps/lwe/src/lib/components apps/lwe/src/routes
git commit -m "feat: polish lwe tailwind feedback surfaces"
```

## Self-Review Checklist

- Spec coverage:
  - Tailwind setup → Task 1
  - primitive alignment → Task 2
  - shell + core components migration → Task 3
  - page reintegration → Task 4
  - minimal feedback polish → Task 5
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the plan.
- Scope check:
  - only the approved UI foundation v1 surfaces are migrated
  - no whole-app Tailwind conversion is attempted
  - backend/domain work remains out of scope

## Expected Output of This Plan

When this plan is complete, LWE’s shell and core product components will no longer sit on a mixed hand-written styling system. Instead, the existing UI foundation will rest on one consistent Tailwind + `shadcn-svelte` primitive layer, making future UI work much less ad hoc.
