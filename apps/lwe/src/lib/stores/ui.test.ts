import { afterEach, describe, expect, it } from 'vitest';
import { get } from 'svelte/store';

import { applyInvalidations, needsPageLoad, pageCache, setLibrarySnapshot, setPageStale } from './ui';

const resetCache = () => {
  pageCache.set({
    library: { snapshot: null, detail: null, stale: false },
    workshop: { snapshot: null, detail: null, stale: false },
    desktop: { snapshot: null, detail: null, stale: false },
    settings: { snapshot: null, detail: null, stale: false }
  });
};

describe('ui page cache', () => {
  afterEach(() => {
    resetCache();
  });

  it('marks a page stale without dropping the cached snapshot', () => {
    const snapshot = {
      items: [],
      selectedItemId: null,
      stale: false
    };

    setLibrarySnapshot(snapshot);
    setPageStale('library');

    const cache = get(pageCache);

    expect(cache.library.snapshot).toEqual(snapshot);
    expect(cache.library.stale).toBe(true);
    expect(needsPageLoad('library')).toBe(true);
  });

  it('applies invalidations per page while keeping existing snapshots', () => {
    const snapshot = {
      items: [],
      selectedItemId: null,
      stale: false
    };

    setLibrarySnapshot(snapshot);
    applyInvalidations(['library']);

    const cache = get(pageCache);

    expect(cache.library.snapshot).toEqual(snapshot);
    expect(cache.library.stale).toBe(true);
    expect(cache.workshop.stale).toBe(false);
  });
});
