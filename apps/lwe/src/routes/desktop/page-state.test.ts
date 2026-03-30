import { describe, expect, it } from 'vitest';

import { resolveDesktopPageState } from './page-state';

describe('desktop page state', () => {
  it('surfaces monitor discovery and persistence issues instead of treating the snapshot as empty', () => {
    expect(
      resolveDesktopPageState({
        monitors: [],
        monitorsAvailable: false,
        monitorDiscoveryIssue: 'Monitor discovery is unavailable.',
        persistenceIssue: 'Assignment persistence is unavailable.',
        assignmentsAvailable: false,
        stale: true
      })
    ).toEqual({
      monitorAvailabilityLabel: 'no',
      assignmentAvailabilityLabel: 'no',
      issueMessages: ['Monitor discovery is unavailable.', 'Assignment persistence is unavailable.'],
      emptyMessage: 'Desktop monitor data is currently unavailable in this snapshot.'
    });
  });

  it('keeps the normal empty copy when monitor data is available but no monitors are present', () => {
    expect(
      resolveDesktopPageState({
        monitors: [],
        monitorsAvailable: true,
        monitorDiscoveryIssue: null,
        persistenceIssue: null,
        assignmentsAvailable: true,
        stale: false
      })
    ).toEqual({
      monitorAvailabilityLabel: 'yes',
      assignmentAvailabilityLabel: 'yes',
      issueMessages: [],
      emptyMessage: 'No monitors are available in the current snapshot.'
    });
  });
});
