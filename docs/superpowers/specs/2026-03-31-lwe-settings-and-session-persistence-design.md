# LWE Settings and Session Persistence Design

## Goal

Unify LWE’s user-facing configuration and session persistence so the app no longer splits persistent behavior across a `wayvid`-named settings path and a separate desktop assignment JSON path.

The result should make the current Settings MVP and desktop assignment restore behavior feel like one coherent product system.

## Why This Work Exists

The current state has three obvious product-level problems:

1. the settings path still uses the old `wayvid` naming
2. Settings use TOML but desktop assignment uses a separate JSON file
3. the user’s selected wallpaper/monitor assignment is not obviously part of the same persistent settings/session story

In addition, two UI-level polish issues remain:

- Settings editing should collapse back to the normal page state after save
- Simplified Chinese should be available as a language choice

## Scope

### In scope

- rename the active user config root from `wayvid` to `lwe`
- unify settings persistence and desktop assignment/session persistence under one TOML-based configuration story
- ensure the selected/applied wallpaper assignment is restored from that unified persistence source
- tighten Settings page behavior after successful save
- add Simplified Chinese as an available language option

### Out of scope

- advanced multi-profile settings
- history/versioned config migration framework
- broad settings-center expansion
- unrelated runtime capability work

## Persistence Direction

The user-facing persistence story should become:

```text
LWE config root
└─ TOML-based configuration/session files
```

The main requirement is conceptual coherence, not forcing every fact into one giant file at any cost.

What matters is:

- the path is `lwe`, not `wayvid`
- the format is TOML-based, not split across TOML + JSON
- restore-able desktop assignment state is part of the same user-facing persistence story as Settings

## Configuration Root

The active config root should become:

```text
$XDG_CONFIG_HOME/lwe/
```

or the corresponding fallback under `~/.config/lwe/`.

The old `wayvid` path should no longer be the active target.

## Session Persistence

Desktop assignment/session persistence should move away from a standalone JSON file and into the TOML-based configuration system.

The persisted session facts should stay minimal:

- monitor assignment
- selected wallpaper identity

This does not require a heavy session framework.

The point is that the app’s Settings and restore behavior now belong to one coherent configuration story.

## Restore Behavior

The restored desktop assignment should keep the current “best effort + explicit degraded reporting” semantics:

- restore when monitor and item still exist
- report missing monitor explicitly
- report missing item explicitly

The difference is that the source of truth now lives in the unified TOML persistence path rather than a separate JSON file.

## Settings UI Follow-Through

The current editable Settings page should feel more complete after this work.

### Required polish

- after a successful save, the editing surface should no longer feel “stuck open” in an active editing state
- language options should include Simplified Chinese

The page should still remain an MVP, but it should feel less provisional.

## Success Criteria

This work is successful when:

1. Active user configuration is stored under `lwe`, not `wayvid`.
2. Desktop assignment/session persistence is no longer stored as a separate JSON track.
3. The currently selected/applied wallpaper assignment can be restored from the unified TOML-based persistence path.
4. Settings save behavior feels complete enough for MVP use.
5. Simplified Chinese appears as a real user-selectable language option.

## Non-Goal Reminder

This is not a full settings-system redesign.

It is the phase where LWE’s user-facing persistence story becomes coherent and the Settings MVP stops feeling disconnected from desktop state.
