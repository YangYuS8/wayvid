import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import DialogTestFixture from './dialog-test-fixture.svelte';

describe('Dialog primitive', () => {
  it('renders Bits UI backed dialog content', () => {
    const { body } = render(DialogTestFixture);

    expect(body).toContain('data-dialog-content');
    expect(body).toContain('data-dialog-title');
    expect(body).toContain('aria-label="Example dialog"');
  });
});
