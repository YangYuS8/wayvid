# LWE UI Foundation v1 Design

## Goal

Use `shadcn-svelte` as the UI primitive layer for LWE and upgrade the current shell plus core product components so the application reads as one coherent desktop product rather than a collection of functional pages.

## Scope

This design covers a **medium UI refresh**, not a full visual reinvention.

### In scope

- introduce `shadcn-svelte` as the primitive component foundation
- establish a minimal token/theme baseline
- replace the current top-link shell with a persistent desktop application frame
- unify page chrome for:
  - `Library`
  - `Workshop`
  - `Desktop`
  - `Settings` (lightly)
- rework the most important product-facing components so they share one visual language:
  - `ItemCard`
  - `LibraryDetailPanel`
  - `WorkshopDetailPanel`
  - `DesktopMonitorCard`
  - `CompatibilityPanel`
  - `StatusBadge`
  - `CoverImage`

### Out of scope

- complete visual redesign of every page detail
- advanced animation system
- major information architecture changes
- broad backend/domain changes
- full dark-theme or multi-theme design work

## Design Direction

The intended visual direction remains:

- media-product first
- restrained rather than flashy
- more polished than a developer tool
- less ornamental than a lifestyle app

The UI should feel suitable for:

- browsing wallpaper content
- understanding compatibility and assignment state
- controlling a desktop application with persistent state

## Priority Order

Implementation should follow this order:

1. App shell, navigation, and page frame
2. Core product component appearance
3. Interaction feedback polish

This ensures the first visible improvement is product coherence, not isolated component polish.

## `shadcn-svelte` Role

`shadcn-svelte` should be the primitive component layer only.

It should supply:

- button
- card
- badge
- select
- dialog
- tooltip
- separator
- toast

It should **not** dictate page layouts or replace LWE’s own product composition.

## App Shell

The app shell should move from the current flat link list to a persistent desktop frame.

Recommended shape:

```text
┌────────────┬──────────────────────────────┐
│ Navigation │ Page header                  │
│            ├──────────────────────────────┤
│            │ Main content                 │
│            │                              │
└────────────┴──────────────────────────────┘
```

The shell should provide:

- persistent navigation for `Library / Workshop / Desktop / Settings`
- current-page emphasis
- a reusable page header pattern for title + subtitle + local actions
- unified content surfaces and spacing rules

This should feel like a desktop application shell, not a website navbar.

## Primitive Layer

The first pass should standardize these primitives:

- primary / secondary button
- content card shell
- status badge
- select / dropdown for monitor choice and future controls
- dialog / sheet for local action surfaces
- separator and spacing primitives
- toast / lightweight feedback surfaces

This gives the product a stable UI grammar without forcing a complete component rewrite all at once.

## Core Product Components

These components should be redesigned against the new primitive layer in the first phase.

### `ItemCard`

Should feel like a media item, not a generic grid tile:

- clear cover presentation
- readable title and type hierarchy
- compatibility status and assignment summary presented with stronger rhythm
- one shared visual language across Library and Workshop

### `LibraryDetailPanel`

Should become a polished action-and-status panel:

- compatibility block clearly separated from content metadata
- apply flow controls integrated into a stronger shell
- assignment state readable without turning into a desktop dashboard

### `WorkshopDetailPanel`

Should emphasize:

- status clarity
- richer content hierarchy
- stronger separation between compatibility explanation and actions

### `DesktopMonitorCard`

Should read like a current-state control surface:

- monitor identity clear
- current assignment clear
- degraded/restore state visible but not noisy
- clear actions feel like part of the product language, not debug controls

### `CompatibilityPanel` / `StatusBadge`

Should become the canonical visual expression of compatibility and state, with one consistent hierarchy across Library, Workshop, and Desktop.

## Page Integration

### Library

Library remains the primary media-management surface.

The UI refresh should make it feel:

- content-first
- easy to scan
- more product-like than technical
- still compatible with the thin-frontend model already in place

### Workshop

Workshop remains a discovery/import view.

The UI refresh should improve:

- list/detail visual hierarchy
- compatibility/action readability
- snapshot browsing comfort

### Desktop

Desktop remains a current-state control view.

The refresh should improve:

- card readability
- degraded-state presentation
- clear-action affordance

### Settings

Settings should adopt the new shell and primitive layer, but does not need deep custom design work in this phase.

## Implementation Strategy

The work should be layered like this:

1. add `shadcn-svelte` and the minimum token/theme setup
2. rebuild the global shell and navigation frame
3. rebuild the core product components against the new primitive layer
4. integrate those components into Library / Workshop / Desktop pages
5. then polish feedback surfaces such as dialog / select / toast usage

This preserves the current product flow while materially improving the visual coherence of the app.

## Success Criteria

The first-phase UI foundation is successful when:

- LWE looks like one product rather than four separate pages
- navigation and shell framing feel stable and desktop-native
- Library / Workshop / Desktop clearly share one design language
- the key product components feel intentionally designed rather than ad hoc
- `shadcn-svelte` is present as a primitive foundation without visually taking over the app
- later UI work can continue from a stable shell and component base instead of page-by-page custom styling
