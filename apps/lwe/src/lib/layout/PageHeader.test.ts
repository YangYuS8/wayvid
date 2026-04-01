import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import PageHeader from './PageHeader.svelte';

describe('PageHeader', () => {
  it('uses utility classes for the shared page-header foundation', () => {
    const { body } = render(PageHeader, {
      props: {
        eyebrow: 'Library',
        title: 'Browse wallpapers',
        subtitle: 'Review local projection state and assignments.'
      }
    });

    expect(body).toContain('tracking-[0.24em]');
    expect(body).toContain('sm:grid-cols-[minmax(0,1fr)_auto]');
    expect(body).not.toContain('<style>');
  });
});
