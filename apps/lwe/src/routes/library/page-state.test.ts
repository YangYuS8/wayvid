import { describe, expect, it } from 'vitest';

import { resolveLibraryPageState } from './page-state';

describe('library page state', () => {
  it('surfaces desktop assignment degradation alongside an empty library snapshot', () => {
    expect(
      resolveLibraryPageState({
        items: [],
        selectedItemId: null,
        monitorsAvailable: false,
        monitorDiscoveryIssue: 'Monitor discovery is unavailable.',
        desktopAssignmentIssue: 'Desktop assignments are unavailable.',
        desktopAssignmentsAvailable: false,
        stale: true
      })
    ).toEqual({
      issueMessages: ['Monitor discovery is unavailable.', 'Desktop assignments are unavailable.'],
      emptyMessage:
        'No Library items are available in the current snapshot. Desktop assignment data is currently unavailable.'
    });
  });

  it('uses the neutral empty copy when assignment data is available', () => {
    expect(
      resolveLibraryPageState({
        items: [],
        selectedItemId: null,
        monitorsAvailable: true,
        monitorDiscoveryIssue: null,
        desktopAssignmentIssue: null,
        desktopAssignmentsAvailable: true,
        stale: false
      })
    ).toEqual({
      issueMessages: [],
      emptyMessage: 'No Library items are available in the current snapshot.'
    });
  });
});
