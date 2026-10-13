import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import { resetPreferredLanguage, setPreferredLanguage } from '$lib/i18n';
import LibraryDetailPanel from './LibraryDetailPanel.svelte';
import { libraryDetail } from '$lib/testing/page-fixtures';
import type { ItemType } from '$lib/types';

describe('LibraryDetailPanel', () => {
  afterEach(() => {
    resetPreferredLanguage();
  });

  it.each([
    ['scene', true, 'DP-1', true],
    ['scene', false, 'DP-1', false],
    ['scene', true, '', false],
    ['video', true, 'DP-1', false],
    ['web', false, 'DP-1', false]
  ] as [ItemType, boolean, string, boolean][])('shows the property entry only for an applicable scene and selected monitor: %s/%s/%s', (itemType, applySupported, selectedMonitorId, visible) => {
    const { body } = render(LibraryDetailPanel, {
      props: { detail: { ...libraryDetail, itemType, applySupported }, selectedMonitorId, applyDisabled: false }
    });
    expect(body.includes('data-detail-section="scene-properties"')).toBe(visible);
    if (visible) {
      expect(body).toContain('aria-expanded="false"');
      expect(body).not.toContain('Save and apply');
    }
  });

  it.each(['applyDisabled', 'applying'] as const)('disables scene properties while %s', (flag) => {
    const { body } = render(LibraryDetailPanel, {
      props: { detail: { ...libraryDetail, itemType: 'scene' }, selectedMonitorId: 'DP-1', applyDisabled: false, [flag]: true }
    });
    const entry = body.match(/<button[^>]*aria-label="Scene properties"[^>]*>/)?.[0];
    expect(entry).toBeDefined();
    expect(entry).toContain('disabled');
  });

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
            detail: 'This video item is synchronized locally and available for Library and desktop use.',
            nextStep: 'none',
            nextStepCopy: null
          },
          applySupported: true,
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
            detail: 'This video item is synchronized locally and available for Library and desktop use.',
            nextStep: 'none',
            nextStepCopy: null
          },
          applySupported: true,
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
            detail: 'This video item is synchronized locally and available for Library and desktop use.',
            nextStep: 'none',
            nextStepCopy: null
          },
          applySupported: true,
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

  it('explains and disables apply controls for recognized items without runtime support', () => {
    const { body } = render(LibraryDetailPanel, {
      props: {
        detail: {
          id: 'scene-1',
          title: 'Forest Scene',
          itemType: 'scene',
          coverPath: null,
          source: 'workshop',
          compatibility: {
            badge: 'partially_supported',
            reasonCode: 'recognized_but_runtime_unsupported',
            summaryCopy: 'Recognized, runtime pending',
            headline: 'Recognized, runtime pending',
            detail: 'LWE recognizes this item for Library and compatibility reporting, but its type or resources do not meet the current playback requirements.',
            nextStep: 'wait_for_future_support',
            nextStepCopy: 'This item needs future runtime support before it can be applied.'
          },
          applySupported: false,
          monitorsAvailable: true,
          monitorDiscoveryIssue: null,
          desktopAssignmentIssue: null,
          desktopAssignmentsAvailable: true,
          assignedMonitorLabels: [],
          description: null,
          tags: []
        },
        monitors: [
          {
            monitorId: 'DISPLAY-1',
            displayName: 'Primary',
            resolution: '1920x1080',
            currentWallpaperTitle: null,
            currentCoverPath: null,
            clearSupported: false,
            runtimeStatus: 'idle'
          }
        ],
        selectedMonitorId: 'DISPLAY-1',
        applyDisabled: true
      }
    });

    expect(body).toContain('This Scene wallpaper cannot play with the current resources and features');
    expect(body).toContain('disabled');
  });

  it('enables an experimental Scene apply attempt only when the item is eligible', () => {
    const { body } = render(LibraryDetailPanel, {
      props: {
        detail: {
          id: 'scene-eligible', title: 'Eligible Scene', itemType: 'scene',
          coverPath: null, source: 'workshop',
          compatibility: {
            badge: 'partially_supported', reasonCode: 'scene_experimental',
            summaryCopy: 'Experimental Scene', headline: 'Experimental Scene',
            detail: 'Assets validated', nextStep: 'none', nextStepCopy: null,
            sceneAssessment: {
              runnable: true,
              summary: { layerCount: 1, effectCount: 0, passCount: 1, animated: false },
              issues: []
            }
          },
          applySupported: true, monitorsAvailable: true,
          monitorDiscoveryIssue: null, desktopAssignmentIssue: null,
          desktopAssignmentsAvailable: true, assignedMonitorLabels: [],
          description: null, tags: []
        },
        monitors: [{
          monitorId: 'DISPLAY-1', displayName: 'Primary', resolution: '1920x1080',
          currentWallpaperTitle: null, currentCoverPath: null,
          clearSupported: false, runtimeStatus: 'idle'
        }],
        selectedMonitorId: 'DISPLAY-1', applyDisabled: false
      }
    });
    const buttons = [...body.matchAll(/<button\b([^>]*)>([\s\S]*?)<\/button>/g)];
    const applyButton = buttons.find(([, , content]) => content.replace(/<[^>]*>/g, '').trim() === 'Apply');
    expect(applyButton).toBeDefined();
    expect(applyButton?.[1]).not.toMatch(/\sdisabled(?:\s|=|$)/);
    expect(body).toContain('Experimental Scene');
    expect(body).not.toContain('This Scene wallpaper cannot play');
  });

  it('shows web runtime unsupported apply messaging for non-video items', () => {
    const { body } = render(LibraryDetailPanel, {
      props: {
        detail: {
          id: 'web-1',
          title: 'Interactive Rain Site',
          itemType: 'web',
          coverPath: null,
          source: 'workshop',
          compatibility: {
            badge: 'partially_supported',
            reasonCode: 'recognized_but_runtime_unsupported',
            summaryCopy: 'Recognized for reporting only',
            headline: 'Runtime unsupported',
            detail:
              'LWE recognizes this web item for Library and compatibility reporting, but its type or resources do not meet the current playback requirements.',
            nextStep: 'wait_for_future_support',
            nextStepCopy: 'This item needs future runtime support before it can be applied.'
          },
          applySupported: false,
          monitorsAvailable: true,
          monitorDiscoveryIssue: null,
          desktopAssignmentIssue: null,
          desktopAssignmentsAvailable: true,
          assignedMonitorLabels: [],
          description: null,
          tags: []
        },
        monitors: [
          {
            monitorId: 'DISPLAY-1',
            displayName: 'Primary',
            resolution: '1920x1080',
            currentWallpaperTitle: null,
            currentCoverPath: null,
            clearSupported: false,
            runtimeStatus: 'idle'
          }
        ],
        selectedMonitorId: 'DISPLAY-1',
        applyDisabled: true
      }
    });

    expect(body).toContain('>Web<');
    expect(body).toContain('Runtime unsupported');
    expect(body).toContain('compatibility reporting');
    expect(body).toContain('type or resources do not meet the current playback requirements');
    expect(body).toContain('disabled');
    expect(body).not.toContain('recognized_but_runtime_unsupported');
  });

  it('renders apply errors inline without replacing the populated detail layout', () => {
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
            detail: 'This video item is synchronized locally and available for Library and desktop use.',
            nextStep: 'none',
            nextStepCopy: null
          },
          applySupported: true,
          monitorsAvailable: true,
          monitorDiscoveryIssue: null,
          desktopAssignmentIssue: null,
          desktopAssignmentsAvailable: true,
          assignedMonitorLabels: ['Primary'],
          description: null,
          tags: []
        },
        applyError: 'Unable to refresh the Library snapshot.'
      }
    });

    expect(body).toContain('Unable to refresh the Library snapshot.');
    expect(body).toContain('data-detail-layout="compact-vertical"');
    expect(body).toContain('data-detail-section="actions"');
    expect(body).not.toContain('Select a Library item to inspect its current detail payload.');
  });

  it('uses a compact vertical detail structure for populated detail', () => {
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
            detail: 'This video item is synchronized locally and available for Library and desktop use.',
            nextStep: 'none',
            nextStepCopy: null
          },
          applySupported: true,
          monitorsAvailable: true,
          monitorDiscoveryIssue: null,
          desktopAssignmentIssue: null,
          desktopAssignmentsAvailable: true,
          assignedMonitorLabels: ['Primary'],
          description: 'Dense detail flow.',
          tags: ['forest']
        }
      }
    });

    expect(body).toContain('data-detail-layout="compact-vertical"');
    expect(body).toContain('data-detail-section="header"');
    expect(body).toContain('data-detail-section="quick-status"');
    expect(body).toContain('data-detail-section="actions"');
    expect(body).toContain('data-detail-section="cover"');
    expect(body).toContain('data-detail-section="compatibility"');
    expect(body).toContain('data-detail-section="metadata"');
    expect(body).not.toContain('lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]');
    expect(body.indexOf('data-detail-section="header"')).toBeLessThan(
      body.indexOf('data-detail-section="quick-status"')
    );
    expect(body.indexOf('data-detail-section="quick-status"')).toBeLessThan(
      body.indexOf('data-detail-section="actions"')
    );
    expect(body.indexOf('data-detail-section="actions"')).toBeLessThan(
      body.indexOf('data-detail-section="cover"')
    );
    expect(body.indexOf('data-detail-section="cover"')).toBeLessThan(
      body.indexOf('data-detail-section="compatibility"')
    );
    expect(body.indexOf('data-detail-section="compatibility"')).toBeLessThan(
      body.indexOf('data-detail-section="metadata"')
    );
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

  it('localizes source and item type labels from centralized i18n copy', () => {
    setPreferredLanguage('zh-CN');

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
            detail: 'This video item is synchronized locally and available for Library and desktop use.',
            nextStep: 'none',
            nextStepCopy: null
          },
          applySupported: true,
          monitorsAvailable: true,
          monitorDiscoveryIssue: null,
          desktopAssignmentIssue: null,
          desktopAssignmentsAvailable: true,
          assignedMonitorLabels: [],
          description: null,
          tags: []
        }
      }
    });

    expect(body).toContain('创意工坊');
    expect(body).toContain('场景');
    expect(body).toContain('完全支持');
    expect(body).not.toContain('Fully Supported');
    expect(body).not.toContain('>workshop<');
    expect(body).not.toContain('>scene<');
  });
});
