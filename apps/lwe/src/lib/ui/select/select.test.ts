import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import Select from './select.svelte';

describe('Select primitive', () => {
  it('renders a native select with its wrapper styling', () => {
    const { body } = render(Select, {
      props: {
        name: 'monitor',
        value: 'primary'
      }
    });

    expect(body).toContain('data-slot="select"');
    expect(body).toContain('name="monitor"');
    expect(body).toContain('hover:border-slate-300');
  });
});
