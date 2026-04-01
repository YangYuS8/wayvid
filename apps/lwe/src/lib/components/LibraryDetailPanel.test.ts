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
    expect(body).toContain('data-slot="card"');
    expect(body).toContain('Library item');
    expect(body).toContain('lwe-info-banner lwe-wrap-safe');
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

  it('shows a visible primary apply action in the detail path', () => {
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
          assignedMonitorLabels: ['Primary'],
          description: null,
          tags: []
        }
      }
    });

    expect(body).toContain('Apply');
    expect(body).toContain('Apply this item to a monitor');
  });

  it('uses the shared subpanel treatment for the empty detail state', () => {
    const { body } = render(LibraryDetailPanel, {
      props: {
        detail: null,
        snapshot: null,
        loading: false,
        error: null
      }
    });

    expect(body).toContain('Select a Library item to inspect its current detail payload.');
    expect(body).toContain('lwe-subpanel');
    expect(body).toContain('border-dashed');
    expect(body).toContain('role="status"');
  });

  it('uses the shared subpanel treatment for the error detail state', () => {
    const { body } = render(LibraryDetailPanel, {
      props: {
        detail: null,
        snapshot: null,
        loading: false,
        error: 'Unable to load the Library request.'
      }
    });

    expect(body).toContain('Unable to load the Library request.');
    expect(body).toContain('lwe-subpanel');
    expect(body).toContain('lwe-warning-banner lwe-wrap-safe');
    expect(body).toContain('Library detail');
  });
});
