# LWE UI Foundation v2 Tailwind Design

## Goal

Complete the second stage of the LWE UI foundation by introducing Tailwind CSS and migrating the already-refreshed shell and core product components onto a unified Tailwind + `shadcn-svelte` styling system.

## Why This Exists

The current UI foundation has already established:

- a persistent application shell
- refreshed core product components
- a small `shadcn-svelte` primitive layer

But the styling system is still mixed:

- some parts use the primitive layer directionally
- many components still rely on local `<style>` blocks

This leaves LWE with a half-complete design system. The purpose of this phase is to unify that styling foundation without expanding into a full product redesign.

## Scope

### In scope

- add Tailwind CSS to ``
- align the existing primitive layer with Tailwind + `shadcn-svelte`
- migrate the existing UI foundation surfaces to the unified styling system:
  - `AppShell`
  - `PageHeader`
  - `ItemCard`
  - `LibraryDetailPanel`
  - `WorkshopDetailPanel`
  - `DesktopMonitorCard`
  - `CompatibilityPanel`
  - `StatusBadge`
  - `CoverImage`
- integrate the migrated components back into:
  - `Library`
  - `Workshop`
  - `Desktop`
  - `Settings`

### Out of scope

- whole-app redesign beyond the existing UI foundation surface
- full page-by-page style rewrite outside foundation components
- major backend or data-flow changes
- broad animation work
- large theme system expansion

## Design Direction

This phase does not change the previously approved visual direction.

LWE should still feel:

- media-product first
- restrained and coherent
- more polished than a tool
- not like a generic component-library showcase

Tailwind is a delivery mechanism for consistency, not the source of the visual identity.

## Implementation Intent

This phase is about moving from a mixed styling approach to a single system.

### Before

```text
primitive direction exists
+
many local component styles remain
```

### After

```text
primitive layer + shell + core product components
all share one Tailwind/shadcn-based styling system
```

## Tailwind Role

Tailwind should become the primary styling mechanism for the UI foundation layer.

It should be used to unify:

- spacing
- typography scale
- surfaces
- borders
- radius
- shadows
- state colors
- layout composition for the shell and key components

It should **not** be used as an excuse to broadly restyle unrelated parts of the app during this phase.

## Primitive Layer Role

The primitive layer should now become a real styling foundation rather than a partial abstraction.

That means:

- the existing `shadcn-svelte`/primitive setup should remain small
- the current primitives should align with Tailwind conventions
- new component work should use those primitives rather than mixing back into isolated local styles

## Shell and Component Targets

The migration should cover the existing UI foundation surfaces only.

### Shell

- `+layout.svelte`
- `AppShell.svelte`
- `PageHeader.svelte`

### Core product components

- `ItemCard`
- `LibraryDetailPanel`
- `WorkshopDetailPanel`
- `DesktopMonitorCard`
- `CompatibilityPanel`
- `StatusBadge`
- `CoverImage`

### Pages

Pages should be updated only as needed to consume the migrated shell/components cleanly:

- `Library`
- `Workshop`
- `Desktop`
- `Settings`

No additional page architecture work should be introduced.

## Success Criteria

This phase is successful when:

- the shell and core product components no longer rely on a mixed local-style vs primitive-style system
- new UI work has one obvious styling path
- the visual language across Library / Workshop / Desktop feels more unified
- the codebase is better prepared for future UI work without continuing style drift

## Non-Goal Reminder

This is **not** a “convert every component in the app to Tailwind” project.

It is a targeted migration of the approved UI foundation v1 surfaces into a real unified styling system.
