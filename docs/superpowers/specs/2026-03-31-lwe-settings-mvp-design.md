# LWE Settings MVP Design

## Goal

Turn the current read-only `Settings` page into a real MVP settings surface that users can edit, persist, and rely on, with first priority on launch-on-login, language, theme, and Steam integration state.

## Scope

### In scope

- editable `Language`
- editable `Theme`
- editable `Launch on login`
- visible `Steam integration state`
- real persistence and reload behavior

### Out of scope

- advanced compatibility tuning
- advanced desktop policy/rule settings
- logging/debug configuration
- `systemd --user` service management
- broad configuration-center design beyond MVP needs

## Persistence Strategy

Settings persistence should use a dedicated TOML configuration file.

SQLite is unnecessary for this scope because the settings surface is small, human-readable, and benefits from a straightforward file-based schema.

The settings layer should therefore be built around:

- a typed settings snapshot
- a typed settings update input
- a TOML file for persistence
- clear defaults when the file does not exist yet

## Startup Behavior

`Launch on login` in this MVP refers specifically to **graphical session autostart**.

It should not attempt to manage a background service or `systemd --user` unit in this phase.

The implementation should target the conventional Linux graphical-session autostart path so that LWE behaves like a desktop application rather than like a system service.

## Settings Surface

The MVP page should stay simple and grouped.

Recommended sections:

### General

- Language
- Theme

### Startup

- Launch on login

### Steam

- Integration status
- concise explanation of current availability/requirements

This is enough to make Settings genuinely useful without turning it into a large control panel.

## Data Model

The implementation should distinguish between:

- `SettingsSnapshot`
  - the currently effective values shown to the UI
- `SettingsUpdateInput`
  - the values the user is trying to change

This keeps the system explicit and avoids letting the UI infer update semantics from the display model alone.

## UX Expectations

The Settings page should become:

- editable
- persistable
- predictable

If the user changes language, theme, or launch-on-login:

- the change should be stored
- the page should reflect the stored result truthfully
- the next app launch should load the new values

For Steam status:

- the page should tell the user what the current state is
- but this phase does not need to provide broad Steam control actions

## Success Criteria

This Settings MVP is successful when:

1. Users can change language, theme, and launch-on-login from the UI.
2. Those settings are persisted in a TOML file.
3. Reopening the app restores the saved values.
4. `Launch on login` uses graphical-session autostart semantics rather than a background service approach.
5. Steam integration state is visible and understandable on the page.

## Non-Goal Reminder

This is the phase where Settings stops being a static snapshot page.

It is not the phase where LWE gets a full-featured configuration center.
