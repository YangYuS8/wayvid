import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import LibraryPage from './+page.svelte';
import { pageCache, setLibraryDetail, setLibrarySnapshot } from '$lib/stores/ui';

const resetCache = () => {
  pageCache.set({
    library: { snapshot: null, detail: null, stale: false },
    workshop: { snapshot: null, detail: null, stale: false },
    desktop: { snapshot: null, detail: null, stale: false },
    settings: { snapshot: null, detail: null, stale: false }
  });
};

describe('library page render', () => {
  afterEach(() => {
    resetCache();
  });

  it('does not render the split body when no library snapshot is available', () => {
    const { body } = render(LibraryPage);

    expect(body).toContain('Your local library');
    expect(body).toContain('Browse the content you already own or have synchronized onto this machine.');
    expect(body).not.toContain('No Library items are available in the current snapshot.');
    expect(body).not.toContain('Select a Library item to inspect its current detail payload.');
  });

  it('renders assigned monitor labels on cards and in the detail panel', () => {
    setLibrarySnapshot({
      items: [
        {
          id: 'scene-7',
          title: 'Forest Scene',
          itemType: 'scene',
          coverPath: null,
          source: 'workshop',
          compatibility: {
            badge: 'fully_supported',
            reasonCode: 'ready_for_library',
            summaryCopy: 'Ready to use'
          },
          favorite: false,
          assignedMonitorLabels: ['Primary', 'DISPLAY-2 (missing)']
        }
      ],
      selectedItemId: 'scene-7',
      monitorsAvailable: true,
      monitorDiscoveryIssue: null,
      desktopAssignmentIssue: null,
      desktopAssignmentsAvailable: true,
      stale: false
    });

    setLibraryDetail({
      id: 'scene-7',
      title: 'Forest Scene',
      itemType: 'scene',
      coverPath: null,
      source: 'workshop',
      compatibility: {
        badge: 'fully_supported',
        reasonCode: 'ready_for_library',
        summaryCopy: 'Ready to use',
        headline: 'Ready to use',
        detail: 'This item is synchronized locally and available for Library and desktop use.',
        nextStep: 'none',
        nextStepCopy: null
      },
      monitorsAvailable: true,
      monitorDiscoveryIssue: null,
      desktopAssignmentIssue: null,
      desktopAssignmentsAvailable: true,
      assignedMonitorLabels: ['Primary', 'DISPLAY-2 (missing)'],
      description: null,
      tags: []
    });

    const { body } = render(LibraryPage);

    expect(body).toContain('Assigned to');
    expect(body).toContain('Assigned monitors');
    expect(body).toContain('Primary');
    expect(body).toContain('DISPLAY-2 (missing)');
  });
});
