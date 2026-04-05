import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import Button from './button.svelte';

describe('Button primitive', () => {
  it('renders its variant and interaction classes', () => {
    const { body } = render(Button, {
      props: {
        variant: 'secondary',
        'aria-label': 'Open dialog'
      }
    });

    expect(body).toContain('bg-primary');
    expect(body).toContain('text-primary-foreground');
    expect(body).toContain('hover:bg-primary/90');
    expect(body).toContain('focus-visible:ring-2');
    expect(body).toContain('data-slot="button"');
    expect(body).toContain('aria-label="Open dialog"');
  });
});
