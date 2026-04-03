# LWE UI Foundation v2 Tailwind Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce Tailwind CSS and migrate the already-established LWE UI foundation surfaces onto a unified Tailwind + `shadcn-svelte` styling system without expanding into a whole-app redesign.

**Architecture:** This is a style-system consolidation pass. It keeps the current shell structure, component boundaries, and thin-frontend contracts, but replaces the mixed local-`<style>` approach with a coherent Tailwind-based primitive + component layer. The migration targets only the already-approved UI foundation surfaces: shell, page headers, and core product components for Library / Workshop / Desktop / Settings.

**Tech Stack:** Svelte 5, Tauri 2, TypeScript, pnpm, Tailwind CSS, `shadcn-svelte`, Bits UI, Vitest

---

## Scope Note

This plan is intentionally limited to the **UI foundation v1 surfaces**.

It includes:

- Tailwind setup in ``
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

- `src/app.css` - Tailwind entrypoint and the minimal app-level tokens/layers for the foundation
- `tailwind.config.ts` - Tailwind configuration for the LWE shell and primitive layer
- `postcss.config.cjs` - Tailwind/PostCSS integration

### Files to modify

- `package.json` - add Tailwind-related dependencies and scripts if needed
- `components.json` - align the primitive config with Tailwind usage
- `src/routes/+layout.svelte` - wire in the global app stylesheet and shell class usage
- `src/lib/layout/AppShell.svelte`
- `src/lib/layout/PageHeader.svelte`
- `src/lib/ui/button/button.svelte`
- `src/lib/ui/badge/badge.svelte`
- `src/lib/ui/card/card.svelte`
- `src/lib/ui/dialog/*.svelte`
- `src/lib/ui/select/*.svelte`
- `src/lib/ui/separator/separator.svelte`
- `src/lib/components/ItemCard.svelte`
- `src/lib/components/LibraryDetailPanel.svelte`
- `src/lib/components/WorkshopDetailPanel.svelte`
- `src/lib/components/DesktopMonitorCard.svelte`
- `src/lib/components/CompatibilityPanel.svelte`
- `src/lib/components/StatusBadge.svelte`
- `src/lib/components/CoverImage.svelte`
- `src/routes/library/+page.svelte`
- `src/routes/workshop/+page.svelte`
- `src/routes/desktop/+page.svelte`
- `src/routes/settings/+page.svelte`

### Files to inspect while implementing

- `src/routes/+layout.svelte`
- `src/lib/layout/AppShell.svelte`
- `src/lib/components/ItemCard.svelte`
- `src/lib/components/LibraryDetailPanel.svelte`
- `src/lib/components/WorkshopDetailPanel.svelte`
- `src/lib/components/DesktopMonitorCard.svelte`
- `docs/superpowers/specs/2026-03-30-lwe-ui-foundation-v2-tailwind-design.md`

## Task 1: Add Tailwind to the LWE Frontend

**Files:**
- Create: `src/app.css`
- Create: `tailwind.config.ts`
- Create: `postcss.config.cjs`
- Modify: `package.json`
- Modify: `components.json`
- Test: `pnpm --dir  check`

- [ ] **Step 1: Write the failing Tailwind setup check**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
assert Path('src/app.css').exists()
assert Path('tailwind.config.ts').exists()
assert Path('postcss.config.cjs').exists()
print('tailwind config present')
PY
```

Expected: FAIL because the Tailwind setup files do not exist yet.

- [ ] **Step 2: Add Tailwind dependencies and configuration**

Update `package.json` with the minimal Tailwind dependencies needed for Svelte + `shadcn-svelte` integration.

Create `tailwind.config.ts` with a content glob that covers:

- `src/**/*.{svelte,ts}`
- `src/lib/ui/**/*.{svelte,ts}`

Create `postcss.config.cjs` to enable Tailwind and autoprefixer.

Update `components.json` so the primitive layer aligns with the Tailwind setup.

- [ ] **Step 3: Create the app stylesheet**

Create `src/app.css` with the minimal Tailwind layers and any token variables the current shell/components need in this phase.

- [ ] **Step 4: Wire the app stylesheet into the shell**

Update `src/routes/+layout.svelte` to import the new global stylesheet.

- [ ] **Step 5: Run verification and commit**

Run:

```bash
pnpm --dir  install && pnpm --dir  check
```

Expected: PASS

Then:

```bash
git add package.json pnpm-lock.yaml components.json tailwind.config.ts postcss.config.cjs src/app.css src/routes/+layout.svelte
git commit -m "feat: add tailwind foundation to lwe"
```

## Task 2: Align the Primitive Layer With Tailwind

**Files:**
- Modify: `src/lib/ui/button/button.svelte`
- Modify: `src/lib/ui/badge/badge.svelte`
- Modify: `src/lib/ui/card/card.svelte`
- Modify: `src/lib/ui/dialog/*.svelte`
- Modify: `src/lib/ui/select/*.svelte`
- Modify: `src/lib/ui/separator/separator.svelte`
- Test: `pnpm --dir  test`

- [ ] **Step 1: Write the failing primitive-style test**

Add or update a primitive test so it asserts one of the current primitives is now rendered through the Tailwind-based style path rather than the older ad hoc class/style mix.

- [ ] **Step 2: Run the frontend tests to verify the current primitive styling is not yet aligned**

Run:

```bash
pnpm --dir  test
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
pnpm --dir  test && pnpm --dir  check
```

Expected: PASS

Then:

```bash
git add src/lib/ui
git commit -m "refactor: align lwe primitives with tailwind foundation"
```

## Task 3: Migrate the Shell and Core Product Components

**Files:**
- Modify: `src/lib/layout/AppShell.svelte`
- Modify: `src/lib/layout/PageHeader.svelte`
- Modify: `src/lib/components/ItemCard.svelte`
- Modify: `src/lib/components/LibraryDetailPanel.svelte`
- Modify: `src/lib/components/WorkshopDetailPanel.svelte`
- Modify: `src/lib/components/DesktopMonitorCard.svelte`
- Modify: `src/lib/components/CompatibilityPanel.svelte`
- Modify: `src/lib/components/StatusBadge.svelte`
- Modify: `src/lib/components/CoverImage.svelte`
- Test: `pnpm --dir  test`

- [ ] **Step 1: Write the failing component-foundation test**

Add or update component tests so they assert the new shell/components consume the Tailwind/styled primitive layer rather than their old local `<style>` grammar.

- [ ] **Step 2: Run tests to verify the current components are still using the mixed style system**

Run:

```bash
pnpm --dir  test
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
pnpm --dir  test && pnpm --dir  check
```

Expected: PASS

Then:

```bash
git add src/lib/layout src/lib/components
git commit -m "refactor: migrate lwe shell and components to tailwind"
```

## Task 4: Re-integrate the Migrated Components Into the Core Pages

**Files:**
- Modify: `src/routes/library/+page.svelte`
- Modify: `src/routes/workshop/+page.svelte`
- Modify: `src/routes/desktop/+page.svelte`
- Modify: `src/routes/settings/+page.svelte`
- Test: `pnpm --dir  test`

- [ ] **Step 1: Write the failing page-integration check**

Add or update page render tests so they assert the pages are using the migrated shell/components without regressing current state/interaction rendering.

- [ ] **Step 2: Run tests to verify integration still needs alignment**

Run:

```bash
pnpm --dir  test
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
pnpm --dir  test && pnpm --dir  check
```

Expected: PASS

Then:

```bash
git add src/routes
git commit -m "feat: integrate tailwind ui foundation into core pages"
```

## Task 5: Polish the Minimal Feedback Surfaces Under the Unified System

**Files:**
- Modify: active component/page files as needed for the current monitor-selection / status / degraded-state feedback surfaces
- Test: `pnpm --dir  test`

- [ ] **Step 1: Identify the smallest necessary feedback polish**

Limit this to the places already touched by the UI foundation where Tailwind + primitive alignment clearly benefits feedback/readability.

- [ ] **Step 2: Apply the minimal polish pass**

Adjust only the current feedback surfaces that are already in the UI foundation scope so they fit the unified styling system.

Avoid broad redesign of interaction flows.

- [ ] **Step 3: Run verification and commit**

Run:

```bash
pnpm --dir  test && pnpm --dir  check
```

Expected: PASS

Then:

```bash
git add src/lib/components src/routes
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
