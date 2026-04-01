import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import SelectTestFixture from './select-test-fixture.svelte';

describe('Select primitive', () => {
  it('renders Bits UI backed trigger and items', () => {
    const { body } = render(SelectTestFixture);

    expect(body).toContain('name="monitor"');
    expect(body).toContain('data-select-trigger');
    expect(body).toContain('data-select-item');
    expect(body).toContain('hover:border-slate-300');
  });
});
