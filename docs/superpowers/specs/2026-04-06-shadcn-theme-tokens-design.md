# Shadcn-Svelte Theme Token Alignment Design

## Goal

Make light and dark themes visually distinct by adopting shadcn-svelte official semantic color tokens and applying them consistently across global shell styles. Keep existing app layout and behavior intact while fixing theme parity.

## Scope

In scope:

- Align app-level CSS variables to shadcn-svelte semantic token system
- Provide both `:root` (light) and `.dark` (dark) token sets
- Keep existing theme settings (`light`, `dark`, `system`) but ensure proper runtime class toggling behavior
- Update custom `lwe-*` style blocks to consume semantic token-driven values instead of hardcoded light-only slate values
- Validate behavior on Library, Workshop, Desktop, Settings pages

Out of scope:

- Adding additional named themes beyond light/dark/system
- Refactoring page/component structure
- Changing feature behavior unrelated to theme rendering

## Confirmed Decisions

- Use implementation approach A: fully align to official shadcn semantic token variables for light/dark
- Keep existing theme selection UX; no new theme options this phase

## Architecture

### Theme source of truth

- CSS token declarations in `src/app.css`
  - `:root` for light theme values
  - `.dark` for dark theme values

Tokens covered:

- `--background`, `--foreground`
- `--card`, `--card-foreground`
- `--popover`, `--popover-foreground`
- `--primary`, `--primary-foreground`
- `--secondary`, `--secondary-foreground`
- `--muted`, `--muted-foreground`
- `--accent`, `--accent-foreground`
- `--destructive`, `--destructive-foreground`
- `--border`, `--input`, `--ring`, `--radius`

### Runtime theme application

- Existing settings pipeline remains authoritative
- Theme applier updates `document.documentElement.classList` with `dark` class for effective dark mode
- `system` mode observes `prefers-color-scheme` and updates class at runtime

### Component styling strategy

- Continue using shadcn utility classes (`bg-background`, `text-foreground`, etc.)
- Update custom shell component classes in `src/app.css` to avoid hardcoded light-only colors:
  - `lwe-shell-bg`
  - `lwe-shell-sidebar`
  - `lwe-shell-main`
  - `lwe-panel`
  - `lwe-panel-compact`
  - `lwe-subpanel`
  - `lwe-info-banner`
  - `lwe-warning-banner`
  - `lwe-eyebrow`, `lwe-body-copy`
  - `lwe-nav-link-*`

## Data Flow

1. User changes theme in Settings (`light` / `dark` / `system`)
2. Theme preference persists via existing settings persistence path
3. Frontend theme applier computes effective mode
4. Root HTML class toggles `dark` as needed
5. Tokenized CSS updates app visuals without page reload

## Styling Rules

- Use semantic token mapping over direct slate shades wherever feasible
- Keep contrast levels sufficient in dark mode for text, borders, focus rings, and banners
- Preserve visual hierarchy and spacing while changing color foundations
- Avoid introducing custom ad-hoc color branches if semantic tokens already cover use case

## Validation Plan

### Functional checks

- `light` mode applies immediately
- `dark` mode applies immediately with clear visual difference
- `system` mode follows OS preference changes

### Persistence checks

- Restart retains selected mode
- `system` remains reactive after restart

### UI checks

- Library / Workshop / Desktop / Settings all render with readable contrast in both modes
- Sidebar/main panels/cards/nav states are visually consistent and distinct by theme

### Regression checks

- Theme switching does not affect data loading, search/filter/pagination, or desktop apply behavior

## Risks and Mitigations

- Risk: partial token migration leaves mixed old/new colors in some components
  - Mitigation: audit all `lwe-*` style blocks and replace light-only hardcoded slate values where they drive major surfaces/text

- Risk: dark mode contrast regressions for niche banners/labels
  - Mitigation: include focused contrast check for warning/info labels and helper text classes

- Risk: system mode class handling drifts from settings state
  - Mitigation: keep one theme applier function and test all three theme states end-to-end
