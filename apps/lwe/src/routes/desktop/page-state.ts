import type { DesktopPageSnapshot } from '$lib/types';

type DesktopPageState = {
  monitorAvailabilityLabel: 'yes' | 'no';
  assignmentAvailabilityLabel: 'yes' | 'no';
  issueMessages: string[];
  emptyMessage: string | null;
};

export const resolveDesktopPageState = (snapshot: DesktopPageSnapshot): DesktopPageState => {
  const issueMessages: string[] = [];

  if (snapshot.monitorDiscoveryIssue) {
    issueMessages.push(snapshot.monitorDiscoveryIssue);
  } else if (!snapshot.monitorsAvailable) {
    issueMessages.push('Monitor discovery is currently unavailable.');
  }

  if (snapshot.persistenceIssue) {
    issueMessages.push(snapshot.persistenceIssue);
  } else if (!snapshot.assignmentsAvailable) {
    issueMessages.push('Desktop assignment persistence is currently unavailable.');
  }

  issueMessages.push(...(snapshot.restoreIssues ?? []));

  return {
    monitorAvailabilityLabel: snapshot.monitorsAvailable ? 'yes' : 'no',
    assignmentAvailabilityLabel: snapshot.assignmentsAvailable ? 'yes' : 'no',
    issueMessages,
    emptyMessage: snapshot.monitors.length
      ? null
      : snapshot.monitorsAvailable
        ? 'No monitors are available in the current snapshot.'
        : 'Desktop monitor data is currently unavailable in this snapshot.'
  };
};
