# Shadcn Theme Token Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make light/dark themes visually distinct by adopting shadcn-svelte semantic tokens and applying them consistently across app shell styles and runtime theme switching behavior.

**Architecture:** Define canonical light and dark token sets in global CSS and ensure runtime theme class application (`dark`) is authoritative for all surfaces. Migrate custom `lwe-*` shell classes from hardcoded palette values to semantic token-driven styles so existing pages inherit correct contrast and hierarchy without structural refactors.

**Tech Stack:** SvelteKit, Tailwind/shadcn-svelte token utilities, global CSS (`src/app.css`), existing settings/theme state pipeline

---

## File Map

- Modify: `src/app.css`
  - Add/update official semantic token values for `:root` and `.dark`, migrate custom shell classes to token-aware styles.
- Modify: `src/lib/stores/ui.ts`
  - Ensure theme state updates are observable and can drive class toggling paths cleanly.
- Modify: `src/routes/settings/+page.svelte`
  - Keep existing theme controls but verify proper update flow and no stale UI assumptions.
- Modify: `src/lib/i18n.ts`
  - Update copy only if theme descriptions/help text needs adjustments.
- Modify: `src/routes/library/+page.svelte`
- Modify: `src/routes/workshop/+page.svelte`
- Modify: `src/routes/desktop/+page.svelte`
- Modify: `src/routes/settings/+page.svelte`
  - Only if page-level class fragments contain hardcoded light-only color assumptions that bypass tokenized shell styles.
- Test: `src/routes/library/page-render.test.ts`
- Test: `src/routes/workshop/page-render.test.ts`
- Test: `src/routes/settings/page-render.test.ts`
  - Validate render output remains coherent after theme class/styling updates.

## Task 1: Establish Canonical Shadcn Light/Dark Token Sets

**Files:**
- Modify: `src/app.css`

- [ ] **Step 1: Write failing CSS token parity check**

Create/extend a style snapshot assertion (if available in existing frontend tests) or add a simple parser test verifying both `:root` and `.dark` blocks contain all required semantic variables.

Run:

```bash
pnpm vitest run src/routes/settings/page-render.test.ts
```

Expected: FAIL (or no coverage) before explicit parity assertion is added.

- [ ] **Step 2: Define official semantic light tokens**

In `src/app.css`, set `:root` token values to shadcn-aligned light palette:

- background/foreground
- card/card-foreground
- popover/popover-foreground
- primary/primary-foreground
- secondary/secondary-foreground
- muted/muted-foreground
- accent/accent-foreground
- destructive/destructive-foreground
- border/input/ring/radius

- [ ] **Step 3: Define official semantic dark tokens in `.dark`**

Add `.dark` block with corresponding dark palette values for all tokens above.

- [ ] **Step 4: Set color-scheme behavior to follow active mode**

Ensure `html` / `html.dark` declarations align browser form controls and native rendering with active theme.

- [ ] **Step 5: Run focused checks**

```bash
pnpm vitest run src/routes/settings/page-render.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit Task 1**

```bash
git add src/app.css src/routes/settings/page-render.test.ts
git commit -m "style: align global light and dark semantic tokens"
```

Expected: commit created.

## Task 2: Migrate Custom Shell Classes to Token-Driven Surfaces

**Files:**
- Modify: `src/app.css`

- [ ] **Step 1: Write failing visual regression-style assertions (render smoke)**

Use existing page render tests as smoke coverage to catch accidental markup/style coupling regressions.

Run:

```bash
pnpm vitest run src/routes/library/page-render.test.ts src/routes/workshop/page-render.test.ts src/routes/desktop/page-render.test.ts src/routes/settings/page-render.test.ts
```

Expected: any failures reveal assumptions that need updates.

- [ ] **Step 2: Update shell background and structural containers**

Migrate these classes to token-aware styles:

- `lwe-shell-bg`
- `lwe-shell-sidebar`
- `lwe-shell-main`

Keep atmosphere but ensure dark mode has meaningful contrast differences.

- [ ] **Step 3: Update panel and content surfaces**

Migrate token usage for:

- `lwe-panel`
- `lwe-panel-compact`
- `lwe-subpanel`

- [ ] **Step 4: Update text and utility classes for dark readability**

Adjust classes including:

- `lwe-eyebrow`, `lwe-body-copy`
- `lwe-info-banner`, `lwe-warning-banner`
- `lwe-nav-link`, `lwe-nav-link-active`, `lwe-nav-link-idle`

- [ ] **Step 5: Re-run render smoke tests**

```bash
pnpm vitest run src/routes/library/page-render.test.ts src/routes/workshop/page-render.test.ts src/routes/desktop/page-render.test.ts src/routes/settings/page-render.test.ts
```

Expected: PASS (main workspace tests).

- [ ] **Step 6: Commit Task 2**

```bash
git add src/app.css src/routes/library/page-render.test.ts src/routes/workshop/page-render.test.ts src/routes/desktop/page-render.test.ts src/routes/settings/page-render.test.ts
git commit -m "style: migrate shell surfaces and typography to semantic theme tokens"
```

Expected: commit created.

## Task 3: Harden Runtime Theme Application (Light/Dark/System)

**Files:**
- Modify: `src/lib/stores/ui.ts`
- Modify: `src/routes/settings/+page.svelte`
- Modify: `src/lib/i18n.ts` (only if copy update needed)

- [ ] **Step 1: Write failing test for runtime theme class behavior**

Add/extend test(s) to assert:

- selecting light removes `dark` class
- selecting dark adds `dark` class
- selecting system follows `prefers-color-scheme`

Run targeted frontend tests for settings/theme logic.

- [ ] **Step 2: Implement single authoritative theme applier**

Ensure one path computes effective mode and updates `document.documentElement.classList` and any companion attributes consistently.

- [ ] **Step 3: Ensure system mode reactivity**

Add/verify media query listener wiring for system mode transitions and cleanup.

- [ ] **Step 4: Ensure persistence restoration applies before/at mount**

On app startup, restore saved setting and apply class promptly to avoid flash of wrong theme.

- [ ] **Step 5: Run targeted settings/theme tests**

```bash
pnpm vitest run src/routes/settings/page-render.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

```bash
git add src/lib/stores/ui.ts src/routes/settings/+page.svelte src/lib/i18n.ts src/routes/settings/page-render.test.ts
git commit -m "feat: enforce consistent runtime light dark system theme application"
```

Expected: commit created.

## Task 4: Cross-Page Validation and Fine-Tuning

**Files:**
- Modify (only as needed):
  - `src/routes/library/+page.svelte`
  - `src/routes/workshop/+page.svelte`
  - `src/routes/desktop/+page.svelte`
  - `src/routes/settings/+page.svelte`

- [ ] **Step 1: Identify hardcoded light-only class fragments in pages**

Scan route pages for direct `slate-*` values that break dark contrast and bypass tokenized wrappers.

- [ ] **Step 2: Replace only high-impact offenders with semantic utility classes**

Keep modifications minimal and focused on readability/contrast.

- [ ] **Step 3: Re-run page render tests**

```bash
pnpm vitest run src/routes/library/page-render.test.ts src/routes/workshop/page-render.test.ts src/routes/desktop/page-render.test.ts src/routes/settings/page-render.test.ts
```

Expected: PASS in main workspace context.

- [ ] **Step 4: Commit Task 4**

```bash
git add src/routes/library/+page.svelte src/routes/workshop/+page.svelte src/routes/desktop/+page.svelte src/routes/settings/+page.svelte src/routes/library/page-render.test.ts src/routes/workshop/page-render.test.ts src/routes/desktop/page-render.test.ts src/routes/settings/page-render.test.ts
git commit -m "style: tune route-level contrast for dark theme readability"
```

Expected: commit created if any route edits were required.

## Task 5: Full Verification

**Files:**
- Verify all theme-related files changed in Tasks 1-4

- [ ] **Step 1: Run frontend tests covering touched routes and theme behavior**

```bash
pnpm vitest run src/routes/library/page-render.test.ts src/routes/workshop/page-render.test.ts src/routes/desktop/page-render.test.ts src/routes/settings/page-render.test.ts
```

Expected: PASS (ignoring unrelated external worktree snapshots if any appear in command output).

- [ ] **Step 2: Run frontend lint/type/build checks used by repo**

```bash
pnpm test
```

Expected: PASS or only known unrelated external-worktree failures documented.

- [ ] **Step 3: Manual runtime verification checklist**

Run app and verify:

- Settings `light` applies immediately
- Settings `dark` applies immediately and looks distinct
- Settings `system` follows OS mode changes
- Restart preserves selected mode
- Library/Workshop/Desktop/Settings remain readable and usable in both modes

- [ ] **Step 4: Final integration commit (only if prior tasks were not committed individually)**

```bash
git add -A
git commit -m "feat: adopt shadcn semantic light dark theme tokens across app shell"
```

Expected: commit created if needed.

## Plan Self-Review

- Spec coverage: token alignment, runtime light/dark/system behavior, shell class migration, and cross-page contrast validation are mapped to explicit tasks.
- Placeholder scan: no TODO/TBD placeholders remain; each task includes concrete files and commands.
- Consistency check: all tasks reference the same semantic token model and avoid introducing extra theme variants outside scope.
