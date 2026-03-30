import type { LibraryItemDetail, LibraryPageSnapshot } from '$lib/types';

type LibraryAvailabilitySource = Pick<
  LibraryPageSnapshot,
  'monitorsAvailable' | 'monitorDiscoveryIssue' | 'desktopAssignmentsAvailable' | 'desktopAssignmentIssue'
>;

type LibraryPageState = {
  issueMessages: string[];
  emptyMessage: string | null;
};

export const resolveLibraryAvailabilityIssues = (
  source: LibraryAvailabilitySource | LibraryItemDetail
): string[] => {
  const issueMessages: string[] = [];

  if (source.monitorDiscoveryIssue) {
    issueMessages.push(source.monitorDiscoveryIssue);
  } else if (!source.monitorsAvailable) {
    issueMessages.push('Monitor discovery is currently unavailable.');
  }

  if (source.desktopAssignmentIssue) {
    issueMessages.push(source.desktopAssignmentIssue);
  } else if (!source.desktopAssignmentsAvailable) {
    issueMessages.push('Desktop assignments are currently unavailable.');
  }

  return issueMessages;
};

export const resolveLibraryPageState = (snapshot: LibraryPageSnapshot): LibraryPageState => {
  const issueMessages = resolveLibraryAvailabilityIssues(snapshot);
  let emptyMessage: string | null = null;

  if (!snapshot.items.length) {
    emptyMessage = 'No Library items are available in the current snapshot.';

    if (!snapshot.desktopAssignmentsAvailable) {
      emptyMessage += ' Desktop assignment data is currently unavailable.';
    } else if (!snapshot.monitorsAvailable) {
      emptyMessage += ' Monitor discovery is currently unavailable.';
    }
  }

  return {
    issueMessages,
    emptyMessage
  };
};
