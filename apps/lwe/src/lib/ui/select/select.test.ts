import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import SelectTestFixture from './select-test-fixture.svelte';

describe('Select primitive', () => {
  it('renders Tailwind-backed trigger and menu items through the primitive slot layer', () => {
    const { body } = render(SelectTestFixture);

    expect(body).toContain('name="monitor"');
    expect(body).toContain('data-slot="select-trigger"');
    expect(body).toContain('data-slot="select-content"');
    expect(body).toContain('data-slot="select-item"');
    expect(body).toContain('border-input bg-background');
    expect(body).toContain('hover:bg-accent');
    expect(body).toContain('hover:text-accent-foreground');
  });
});
