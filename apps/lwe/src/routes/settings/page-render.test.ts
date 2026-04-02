import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import SettingsPage from './+page.svelte';
import { pageCache, setSettingsSnapshot } from '$lib/stores/ui';

const resetCache = () => {
  pageCache.set({
    library: { snapshot: null, detail: null, stale: false },
    workshop: { snapshot: null, detail: null, stale: false },
    desktop: { snapshot: null, detail: null, stale: false },
    settings: { snapshot: null, detail: null, stale: false }
  });
};

describe('settings page render', () => {
  afterEach(() => {
    resetCache();
  });

  it('renders editable controls and steam status instead of a static snapshot table', () => {
    setSettingsSnapshot({
      language: 'en',
      theme: 'system',
      launchOnLogin: true,
      launchOnLoginAvailable: true,
      steamRequired: true,
      steamStatusMessage: 'Steam is required to launch Wallpaper Engine content.',
      stale: false
    });

    const { body } = render(SettingsPage);

    expect(body).toContain('Language');
    expect(body).toContain('Theme');
    expect(body).toContain('Launch on login');
    expect(body).toContain('Steam integration');
    expect(body).toContain('Steam is required to launch Wallpaper Engine content.');
    expect(body).toContain('Save changes');
    expect(body).toContain('English');
    expect(body).toContain('Follow system theme');
    expect(body).toContain('Launch on login:');
    expect(body).toContain('enabled');
    expect(body).not.toContain('Rust backend');
    expect(body).not.toContain('backend-owned settings file');
    expect(body).not.toContain('Snapshot stale');
  });

  it('describes launch-on-login as a saved preference when autostart is unavailable', () => {
    setSettingsSnapshot({
      language: 'system',
      theme: 'dark',
      launchOnLogin: true,
      launchOnLoginAvailable: false,
      steamRequired: false,
      steamStatusMessage: 'Steam is optional for the current setup.',
      stale: false
    });

    const { body } = render(SettingsPage);

    expect(body).toContain('Launch-on-login is currently unavailable on this machine.');
    expect(body).toContain('Saved launch preference:');
    expect(body).toContain('Prefer enabled when available');
    expect(body).not.toContain('Launch on login:');
  });
});
