import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import WorkshopPage from './+page.svelte';
import { pageCache } from '$lib/stores/ui';

const resetCache = () => {
  pageCache.set({
    library: { snapshot: null, detail: null, stale: false },
    workshop: { snapshot: null, detail: null, stale: false },
    desktop: { snapshot: null, detail: null, stale: false },
    settings: { snapshot: null, detail: null, stale: false }
  });
};

describe('workshop page render', () => {
  afterEach(() => {
    resetCache();
  });

  it('does not render the split body when no workshop snapshot is available', () => {
    const { body } = render(WorkshopPage);

    expect(body).toContain('Local Workshop sync');
    expect(body).toContain('Review the current Steam Workshop items synced into Wayvid from this machine. This is not a full online Workshop browser.');
    expect(body).not.toContain('No Workshop items are available in the current snapshot.');
    expect(body).not.toContain('Select a Workshop item to inspect its current detail payload.');
  });
});
