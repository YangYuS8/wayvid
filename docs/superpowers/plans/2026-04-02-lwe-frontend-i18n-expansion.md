# LWE Frontend i18n Expansion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand the lightweight frontend i18n layer so the main LWE shell and its primary shared components render coherent Simplified Chinese UI.

**Architecture:** Keep the existing `src/lib/i18n.ts` module and grow it into a centralized, domain-organized dictionary. Update each primary route and its directly rendered shared components to read from the shared copy object, while intentionally leaving backend-generated natural-language messages untranslated.

**Tech Stack:** Svelte 5, SvelteKit, Vitest, TypeScript, existing lightweight store-based i18n module

---

## File Map

- Modify: `src/lib/i18n.ts`
  - Expand the dictionary with `library`, `workshop`, `desktop`, and `components` sections.
  - Add any small interpolation helpers needed for route and component copy.
- Modify: `src/routes/library/+page.svelte`
  - Replace route-owned hard-coded English copy with i18n lookups.
- Modify: `src/routes/workshop/+page.svelte`
  - Replace route-owned hard-coded English copy with i18n lookups.
- Modify: `src/routes/desktop/+page.svelte`
  - Replace route-owned hard-coded English copy with i18n lookups.
- Modify: `src/routes/settings/+page.svelte`
  - Normalize any remaining literal copy so the page stays fully dictionary-backed.
- Modify: `src/lib/layout/AppShell.svelte`
  - Keep shell translation wiring aligned with the expanded dictionary shape.
- Modify: `src/lib/components/LibraryDetailPanel.svelte`
  - Translate route-facing library action and detail copy.
- Modify: `src/lib/components/WorkshopDetailPanel.svelte`
  - Translate workshop detail panel copy.
- Modify: `src/lib/components/DesktopMonitorCard.svelte`
  - Translate monitor status/detail labels owned by the component.
- Modify: `src/lib/components/ItemCard.svelte`
  - Translate any remaining user-facing hard-coded labels if present.
- Test: `src/lib/layout/AppShell.test.ts`
  - Keep shell language assertions aligned with the final dictionary shape.
- Test: `src/routes/settings/page-render.test.ts`
  - Keep Settings zh-CN assertions passing.
- Test: `src/routes/desktop/page-render.test.ts`
  - Add/assert zh-CN desktop labels.
- Test: `src/routes/library/page-render.test.ts`
  - Add route-level zh-CN rendering coverage.
- Test: `src/routes/workshop/page-render.test.ts`
  - Add route-level zh-CN rendering coverage.
- Test: component render tests under `src/lib/components/` as needed
  - Extend existing tests or add focused render tests where a shared component owns visible copy.

### Task 1: Expand the i18n Dictionary Shape First

**Files:**
- Modify: `src/lib/i18n.ts`
- Test: `src/lib/layout/AppShell.test.ts`
- Test: `src/routes/settings/page-render.test.ts`

- [ ] **Step 1: Write the failing dictionary-shape assertions**

Update tests so they require the new domain coverage to exist through actual rendered output. Add or keep assertions for these concrete strings:

```ts
expect(body).toContain('内容库');
expect(body).toContain('创意工坊');
expect(body).toContain('桌面');
expect(body).toContain('设置');
expect(body).toContain('应用偏好');
expect(body).toContain('当前设置');
```

- [ ] **Step 2: Run the focused tests to verify the new coverage fails before implementation**

Run:

```bash
pnpm --dir "" test -- --run src/lib/layout/AppShell.test.ts src/routes/settings/page-render.test.ts
```

Expected: at least one failure showing missing or mismatched copy from the expanded dictionary shape.

- [ ] **Step 3: Expand `src/lib/i18n.ts` with the new top-level sections**

Keep the existing store pattern. Extend the dictionary so both locales have at least this structure:

```ts
const dictionaries = {
  en: {
    appShell: { ... },
    library: { ... },
    workshop: { ... },
    desktop: { ... },
    settings: { ... },
    components: { ... }
  },
  'zh-CN': {
    appShell: { ... },
    library: { ... },
    workshop: { ... },
    desktop: { ... },
    settings: { ... },
    components: { ... }
  }
} as const;
```

Add only small helpers, for example:

```ts
export const labelWithName = (prefix: string, name: string) => `${prefix} ${name}`;
```

Do not add a generic formatting framework.

- [ ] **Step 4: Run the same focused tests to verify the dictionary-backed shell still passes**

Run:

```bash
pnpm --dir "" test -- --run src/lib/layout/AppShell.test.ts src/routes/settings/page-render.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the dictionary-shape foundation**

Run:

```bash
git add src/lib/i18n.ts src/lib/layout/AppShell.test.ts src/routes/settings/page-render.test.ts
git commit -m "feat: expand frontend i18n dictionary structure"
```

### Task 2: Translate the Library Route and Its Primary Detail UI

**Files:**
- Modify: `src/routes/library/+page.svelte`
- Modify: `src/lib/components/LibraryDetailPanel.svelte`
- Modify: `src/lib/components/ItemCard.svelte` (if it owns visible hard-coded labels)
- Modify: `src/lib/i18n.ts`
- Test: `src/routes/library/page-render.test.ts`
- Test: component tests under `src/lib/components/`

- [ ] **Step 1: Write the failing Library zh-CN render test**

Add or extend `src/routes/library/page-render.test.ts` with a case that sets `zh-CN` and expects concrete Chinese route-owned copy, for example:

```ts
expect(body).toContain('内容库');
expect(body).toContain('本地内容库');
expect(body).toContain('正在加载内容库快照…');
expect(body).toContain('当前快照中没有可用的内容项。');
```

If the detail panel is covered in the same render path, also assert one or two owned labels such as:

```ts
expect(body).toContain('操作');
expect(body).toContain('应用');
```

- [ ] **Step 2: Run the focused Library test to verify it fails**

Run:

```bash
pnpm --dir "" test -- --run src/routes/library/page-render.test.ts
```

Expected: FAIL on untranslated English strings.

- [ ] **Step 3: Implement minimal Library i18n wiring**

Update `src/routes/library/+page.svelte` to read from `$copy.library`, matching the current Settings pattern:

```ts
import { copy } from '$lib/i18n';

const readError = (error: unknown) =>
  error instanceof Error ? error.message : $copy.library.requestError;
```

Move route text into dictionary-backed lookups, including page header, loading text, empty-state text, and route-owned selection/apply labels.

Update `LibraryDetailPanel.svelte` so any stable labels it owns also come from `$copy.components.libraryDetail` or a similarly scoped section.

- [ ] **Step 4: Re-run the focused Library tests**

Run:

```bash
pnpm --dir "" test -- --run src/routes/library/page-render.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the Library i18n slice**

Run:

```bash
git add src/lib/i18n.ts src/routes/library/+page.svelte src/lib/components/LibraryDetailPanel.svelte src/lib/components/ItemCard.svelte src/routes/library/page-render.test.ts
git commit -m "feat: localize library frontend shell"
```

### Task 3: Translate the Workshop Route and Its Detail UI

**Files:**
- Modify: `src/routes/workshop/+page.svelte`
- Modify: `src/lib/components/WorkshopDetailPanel.svelte`
- Modify: `src/lib/i18n.ts`
- Test: `src/routes/workshop/page-render.test.ts`
- Test: component tests under `src/lib/components/`

- [ ] **Step 1: Write the failing Workshop zh-CN render test**

Add or extend `src/routes/workshop/page-render.test.ts` with concrete Chinese assertions such as:

```ts
expect(body).toContain('创意工坊');
expect(body).toContain('本地创意工坊同步');
expect(body).toContain('刷新目录');
expect(body).toContain('当前快照中没有可用的创意工坊项目。');
```

- [ ] **Step 2: Run the focused Workshop test to verify it fails**

Run:

```bash
pnpm --dir "" test -- --run src/routes/workshop/page-render.test.ts
```

Expected: FAIL on untranslated route copy.

- [ ] **Step 3: Implement minimal Workshop i18n wiring**

Update `src/routes/workshop/+page.svelte` to read route-owned copy from `$copy.workshop`, including:

```ts
<PageHeader
  eyebrow={$copy.workshop.pageTitle}
  title={$copy.workshop.headerTitle}
  subtitle={$copy.workshop.headerSubtitle}
/>
```

Translate the refresh button, loading copy, request fallback copy, and empty state. Update `WorkshopDetailPanel.svelte` for any stable labels it owns.

- [ ] **Step 4: Re-run the focused Workshop tests**

Run:

```bash
pnpm --dir "" test -- --run src/routes/workshop/page-render.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the Workshop i18n slice**

Run:

```bash
git add src/lib/i18n.ts src/routes/workshop/+page.svelte src/lib/components/WorkshopDetailPanel.svelte src/routes/workshop/page-render.test.ts
git commit -m "feat: localize workshop frontend shell"
```

### Task 4: Translate the Desktop Route and Monitor Card Copy

**Files:**
- Modify: `src/routes/desktop/+page.svelte`
- Modify: `src/lib/components/DesktopMonitorCard.svelte`
- Modify: `src/lib/i18n.ts`
- Test: `src/routes/desktop/page-render.test.ts`
- Test: component tests under `src/lib/components/`

- [ ] **Step 1: Write the failing Desktop zh-CN render test**

Extend `src/routes/desktop/page-render.test.ts` so a zh-CN case asserts concrete Chinese route-owned copy such as:

```ts
expect(body).toContain('桌面');
expect(body).toContain('显示器外壳');
expect(body).toContain('视图');
expect(body).toContain('全部输出');
expect(body).toContain('当前显示器');
expect(body).toContain('缺失恢复项');
```

Also assert at least one translated monitor-card label if that component owns visible copy:

```ts
expect(body).toContain('当前项目');
expect(body).toContain('恢复状态');
```

- [ ] **Step 2: Run the focused Desktop test to verify it fails**

Run:

```bash
pnpm --dir "" test -- --run src/routes/desktop/page-render.test.ts
```

Expected: FAIL on untranslated desktop copy.

- [ ] **Step 3: Implement minimal Desktop i18n wiring**

Update `src/routes/desktop/+page.svelte` so route-owned labels come from `$copy.desktop`, including filter labels, summary labels, loading text, empty states, and footer copy.

Update `DesktopMonitorCard.svelte` so stable labels it owns come from `$copy.components.desktopMonitorCard`.

Do not translate backend-provided `actionMessage`, `actionError`, or runtime reason strings.

- [ ] **Step 4: Re-run the focused Desktop tests**

Run:

```bash
pnpm --dir "" test -- --run src/routes/desktop/page-render.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the Desktop i18n slice**

Run:

```bash
git add src/lib/i18n.ts src/routes/desktop/+page.svelte src/lib/components/DesktopMonitorCard.svelte src/routes/desktop/page-render.test.ts
git commit -m "feat: localize desktop frontend shell"
```

### Task 5: Normalize Shared Fallbacks and Run Full Frontend Verification

**Files:**
- Modify: `src/routes/settings/+page.svelte`
- Modify: `src/routes/+layout.svelte`
- Modify: any touched route/component test files from earlier tasks
- Test: `src/lib/layout/AppShell.test.ts`
- Test: `src/routes/settings/page-render.test.ts`
- Test: `src/routes/library/page-render.test.ts`
- Test: `src/routes/workshop/page-render.test.ts`
- Test: `src/routes/desktop/page-render.test.ts`

- [ ] **Step 1: Add any final failing tests for mixed-language gaps discovered during route work**

If any remaining route/component still renders obvious hard-coded English during zh-CN mode, add a concrete assertion before fixing it. Use the existing route render tests instead of creating new broad snapshot suites.

- [ ] **Step 2: Run the affected frontend tests to verify the last gap fails**

Run the smallest focused command for the touched test file, for example:

```bash
pnpm --dir "" test -- --run src/routes/settings/page-render.test.ts
```

Expected: FAIL on the remaining mixed-language gap.

- [ ] **Step 3: Implement the minimal final cleanup**

Normalize any remaining route-owned literals or type-safe language wiring without broad refactoring. Keep the final state aligned with the lightweight store-driven model already in place.

- [ ] **Step 4: Run full frontend verification**

Run:

```bash
pnpm --dir "" test
pnpm --dir "" check
```

Expected: all frontend tests pass and `svelte-check` reports `0 errors and 0 warnings`.

- [ ] **Step 5: Commit the completed frontend i18n expansion**

Run:

```bash
git add src/lib/i18n.ts src/routes/+layout.svelte src/routes/settings/+page.svelte src/routes/library/+page.svelte src/routes/workshop/+page.svelte src/routes/desktop/+page.svelte src/lib/layout/AppShell.svelte src/lib/components/LibraryDetailPanel.svelte src/lib/components/WorkshopDetailPanel.svelte src/lib/components/DesktopMonitorCard.svelte src/lib/components/ItemCard.svelte src/lib/layout/AppShell.test.ts src/routes/settings/page-render.test.ts src/routes/library/page-render.test.ts src/routes/workshop/page-render.test.ts src/routes/desktop/page-render.test.ts
git commit -m "feat: localize the primary frontend shell"
```

## Plan Self-Review

- Spec coverage check: this plan covers dictionary expansion, primary routes, high-frequency shared components, fallback behavior, and route/component render tests for zh-CN.
- Placeholder scan: no `TODO`, `TBD`, or "similar to" references remain.
- Type consistency check: the plan consistently uses the existing `copy`-store approach, route render tests, and `pnpm --dir "" test/check` verification commands.
