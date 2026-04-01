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

  it('uses utility-based shell classes instead of the older local style block', () => {
    const { body } = render(AppShell, {
      props: {
        currentPath: '/library'
      }
    });

    expect(body).toContain('bg-slate-950/90');
    expect(body).toContain('lg:grid-cols-[minmax(248px,292px)_minmax(0,1fr)]');
    expect(body).not.toContain('<style>');
  });
});
