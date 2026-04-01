import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import Dialog from './dialog.svelte';

describe('Dialog primitive', () => {
  it('renders a native dialog with the requested open state', () => {
    const { body } = render(Dialog, {
      props: {
        open: true,
        'aria-label': 'Example dialog'
      }
    });

    expect(body).toContain('data-slot="dialog"');
    expect(body).toContain('open');
    expect(body).toContain('aria-label="Example dialog"');
  });
});
