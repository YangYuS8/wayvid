import { afterEach, describe, expect, it } from 'vitest';
import { get } from 'svelte/store';

import {
  applyInvalidations,
  isSelectedItem,
  needsPageLoad,
  pageCache,
  setLibraryDetailIfSelected,
  setLibrarySnapshot,
  setPageStale,
  setSelectedItem
} from './ui';

const compatibilitySummary = {
  badge: 'fully_supported' as const,
  reasonCode: 'ready_for_library',
  summaryCopy: 'Ready to use'
};

const compatibilityDetail = {
  ...compatibilitySummary,
  headline: 'Ready to use',
  detail: 'This item is synchronized locally and available for Library and desktop use.',
  nextStep: 'none' as const,
  nextStepCopy: null
};

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

  it('ignores stale detail responses for a previous library selection', () => {
    setLibrarySnapshot({
      items: [
        {
          id: 'a',
          title: 'A',
          itemType: 'scene',
          coverPath: null,
          source: 'workshop',
          compatibility: compatibilitySummary,
          favorite: false
        },
        {
          id: 'b',
          title: 'B',
          itemType: 'scene',
          coverPath: null,
          source: 'workshop',
          compatibility: compatibilitySummary,
          favorite: false
        }
      ],
      selectedItemId: null,
      stale: false
    });

    setSelectedItem('library', 'a');
    setSelectedItem('library', 'b');
    setLibraryDetailIfSelected(
      {
        id: 'a',
        title: 'A',
        itemType: 'scene',
        coverPath: null,
        source: 'workshop',
        compatibility: compatibilityDetail,
        description: null,
        tags: []
      },
      'a'
    );

    const cache = get(pageCache);

    expect(isSelectedItem('library', 'a')).toBe(false);
    expect(isSelectedItem('library', 'b')).toBe(true);
    expect(cache.library.detail).toBeNull();
  });
});
