# LWE Usability Pass v1 Design

## Goal

Improve the first-use clarity and day-to-day usability of the current LWE shell by clarifying page roles, making the apply action obvious and fast, and tightening the detail layout so it supports real use instead of feeling like a stretched prototype panel.

## Why This Work Exists

LWE now has:

- a coherent shell
- real Workshop, compatibility, and desktop-state foundations
- a unified UI system

But it still falls short of “immediately understandable and usable” because:

- `Library` and `Workshop` feel too similar
- the main `Apply` action is not obvious enough
- detail panels consume too much space for too little information density

This design is about improving usability, not adding broad new features.

## Scope

### In scope

- clarify `Library` vs `Workshop` page semantics
- make `Apply` visibly discoverable and efficient
- redesign the detail panel layout to use space better

### Out of scope

- deeper Settings work
- true online Workshop browsing
- advanced desktop policy/rules
- new runtime capabilities

## Page Semantics

### Library

`Library` should read as:

- the place for content the user already has locally
- the primary place to choose content and start using it

### Workshop

The long-term product meaning of `Workshop` is still the Wallpaper Engine ecosystem/discovery surface.

However, for the current implementation stage, the page should be made **explicitly truthful**:

- it is a view over locally synchronized Workshop content
- it is not yet the full online Workshop browser

The page name can remain `Workshop`, but page framing and copy should make its current local-sync role clear.

## Apply Action Strategy

The apply action should be easier to discover without turning every card into a control panel.

### Recommended approach

- a visible primary `Apply` action in the core Library interaction path
- a right-click/context-menu path as a secondary shortcut

### Primary path

The primary action should be visible in the detail panel for the selected item.

This keeps:

- the main flow obvious for new users
- monitor selection close to the main action
- the UI cleaner than stuffing too many controls into every card by default

### Secondary path

The card or item surface may expose a context menu for faster use.

This should be treated as a convenience path, not as the only way to apply a wallpaper.

## Detail Panel Layout

The current detail layout behaves too much like a full-width media detail page and not enough like a compact desktop application side panel.

### Problem

- cover art is too dominant
- metadata and status are too spread out
- text density is poor
- the panel can overflow or feel wasteful at common app sizes

### Direction

Move from a large two-column media-style layout to a more compact vertical information flow.

Recommended structure:

1. Header
   - title
   - type/source
   - badges
2. Quick status
   - assignment state
   - degraded state if relevant
3. Actions
   - Apply
   - monitor selection
   - secondary actions if present
4. Compact cover
5. Compatibility block
6. Metadata block

This makes the panel feel more like an action-oriented desktop detail surface and less like an oversized content showcase.

## UX Priorities

The first usability pass should optimize for these user questions:

- What is this page for?
- Is this wallpaper already available locally?
- Can I apply it right now?
- Where do I click to use it?
- What is currently applied to my desktop?

If the interface answers those clearly, the pass is successful.

## Success Criteria

This usability pass is successful when:

- users can clearly distinguish `Library` from the current local-sync `Workshop` view
- `Apply` is easy to find without relying on hidden interaction only
- the detail panel no longer wastes space or feels structurally oversized
- the product feels more obviously usable, even before further feature expansion
