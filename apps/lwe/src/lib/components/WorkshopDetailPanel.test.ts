import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import WorkshopDetailPanel from './WorkshopDetailPanel.svelte';

describe('WorkshopDetailPanel', () => {
  it('uses the shared subpanel treatment for the empty detail state', () => {
    const { body } = render(WorkshopDetailPanel, {
      props: {
        detail: null,
        loading: false,
        error: null,
        openInSteam: null
      }
    });

    expect(body).toContain('Select a Workshop item to inspect its current detail payload.');
    expect(body).toContain('lwe-subpanel');
    expect(body).toContain('border-dashed');
    expect(body).toContain('role="status"');
  });
});
