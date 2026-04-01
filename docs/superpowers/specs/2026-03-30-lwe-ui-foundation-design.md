# LWE UI Foundation Design

## Goal

Establish a unified visual and interaction foundation for the active LWE shell using `shadcn-svelte` as the UI primitive layer, without changing the product’s thin-frontend architecture or overextending into broad visual polish.

## Product Direction

The UI foundation should make LWE feel like a real desktop application rather than a collection of functional Svelte pages. The visual direction is:

- media-product first
- restrained rather than flashy
- more polished than a developer tool
- less ornamental than a lifestyle app

This should feel appropriate for:

- browsing wallpaper content
- understanding compatibility
- controlling a desktop stateful application

## Priorities

The first implementation phase should follow this order:

1. App shell, navigation, and base page frame
2. Core component appearance for Library, Workshop, and Desktop
3. Interaction feedback primitives

That means the first goal is not “make every page beautiful.” It is “make the whole app look and behave like one coherent product.”

## Scope

### In scope

- introduce `shadcn-svelte` as the UI primitive layer
- define a minimal visual language for LWE
- replace the current minimal top navigation with a real application shell
- unify page header structure and content containers
- upgrade the visual language of:
  - `Library`
  - `Workshop`
  - `Desktop`
- standardize the most important primitives:
  - button
  - card
  - badge
  - select
  - dialog
  - tooltip
  - separator
  - toast

### Out of scope

- full visual redesign of every page detail
- advanced motion system
- dark-mode/theme-system expansion beyond what the base layer needs
- large empty-state illustration work
- major page information architecture changes
- backend or domain-model changes unrelated to UI integration

## App Shell

The new LWE shell should move away from the current flat top-link structure and become a stable desktop application frame.

Recommended shape:

```text
┌────────────┬──────────────────────────────┐
│ Navigation │ Page header                  │
│            ├──────────────────────────────┤
│            │ Main content                 │
│            │                              │
└────────────┴──────────────────────────────┘
```

This should provide:

- persistent navigation for `Library / Workshop / Desktop / Settings`
- clear current-page emphasis
- a consistent header slot for title + subtitle + local actions
- a unified content surface for page bodies

The shell should feel like a desktop app frame, not like a website navbar.

## Visual Language

LWE should use a restrained media-product language.

### Desired qualities

- clean surfaces
- strong but not loud card hierarchy
- clear spacing rhythm
- predictable typography levels
- status colors that communicate meaning without dominating the page

### Avoid

- neon/futuristic excess
- dashboard-heavy enterprise styling
- generic component-library demo aesthetics

The UI should feel comfortable for browsing content over time, not just for showing that the app works.

## `shadcn-svelte` Role

`shadcn-svelte` should be used as the primitive component layer, not as the source of LWE’s product identity.

Use it to provide:

- button
- card
- badge
- select
- dialog
- tooltip
- separator
- toast

Do **not** treat its example layouts as the product design. LWE still needs its own page composition and media-facing presentation.

## First-Phase Component Targets

The first phase should standardize the shell and the most visible custom components.

### Shell-level

- navigation container
- page header block
- content container
- section cards/panels

### Content-level

- `ItemCard`
- `LibraryDetailPanel`
- `WorkshopDetailPanel`
- `DesktopMonitorCard`
- status badges and compatibility panels

These components do not need a full conceptual rewrite. They need a common visual grammar.

## Page Focus

### Library

Should feel like the primary media-management surface:

- content-first
- quick compatibility understanding
- apply actions clearly available
- assignment state visible without turning into a control dashboard

### Workshop

Should feel like a discovery/import surface:

- snapshot list remains dominant
- detail panel becomes clearer and more product-like
- status and action hierarchy become more legible

### Desktop

Should feel like a current-state control surface:

- monitor cards should look intentional and readable
- degraded state should be legible
- clear actions should feel part of the product language, not debug controls

## Implementation Strategy

The first phase should be done in layers:

1. add `shadcn-svelte` primitives and supporting style/token setup
2. replace the global shell and page frame
3. upgrade the shared components used by Library / Workshop / Desktop
4. then refine interaction feedback within the new shell

This preserves the existing product flows while making the visible application feel coherent.

## Success Criteria

This UI foundation work is successful when:

- LWE reads visually as one product rather than four loosely related pages
- navigation and page framing feel desktop-native and stable
- Library / Workshop / Desktop clearly share one design language
- `shadcn-svelte` is clearly present as a primitive layer without visually taking over the product
- later UI work can build on a stable shell and component base instead of continuing page-by-page ad hoc styling
