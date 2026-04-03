import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import DialogTestFixture from './dialog-test-fixture.svelte';

describe('Dialog primitive', () => {
  it('renders Tailwind-backed dialog content through the primitive slot layer', () => {
    const { body } = render(DialogTestFixture);

    expect(body).toContain('data-slot="dialog-content"');
    expect(body).toContain('data-slot="dialog-overlay"');
    expect(body).toContain('border-border bg-card p-6');
    expect(body).toContain('data-slot="dialog-title"');
    expect(body).toContain('aria-label="Example dialog"');
  });
});
