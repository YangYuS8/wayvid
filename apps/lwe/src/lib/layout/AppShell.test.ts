import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import AppShell from './AppShell.svelte';

describe('AppShell', () => {
  it('renders persistent navigation, current-page emphasis, and a main region', () => {
    const { body } = render(AppShell, {
      props: {
        currentPath: '/desktop'
      }
    });

    expect(body).toContain('Library');
    expect(body).toContain('Workshop');
    expect(body).toContain('Desktop');
    expect(body).toContain('Settings');
    expect(body).toContain('aria-current="page"');
    expect(body).toContain('<main');
    expect(body).not.toContain('<h1');
  });
});
