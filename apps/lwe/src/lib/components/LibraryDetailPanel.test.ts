import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import LibraryDetailPanel from './LibraryDetailPanel.svelte';

describe('LibraryDetailPanel', () => {
  it('surfaces desktop assignment degradation from the current detail payload', () => {
    const { body } = render(LibraryDetailPanel, {
      props: {
        detail: {
          id: 'scene-1',
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
          monitorsAvailable: false,
          monitorDiscoveryIssue: 'Monitor discovery is unavailable.',
          desktopAssignmentIssue: 'Desktop assignments are unavailable.',
          desktopAssignmentsAvailable: false,
          assignedMonitorLabels: [],
          description: null,
          tags: []
        }
      }
    });

    expect(body).toContain('Monitor discovery is unavailable.');
    expect(body).toContain('Desktop assignments are unavailable.');
  });

  it('renders assigned monitor labels from the current detail payload', () => {
    const { body } = render(LibraryDetailPanel, {
      props: {
        detail: {
          id: 'scene-1',
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
        }
      }
    });

    expect(body).toContain('Assigned monitors');
    expect(body).toContain('Primary');
    expect(body).toContain('DISPLAY-2 (missing)');
  });
});
