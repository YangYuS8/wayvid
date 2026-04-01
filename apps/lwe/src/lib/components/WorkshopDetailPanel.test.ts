import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import WorkshopDetailPanel from './WorkshopDetailPanel.svelte';

describe('WorkshopDetailPanel', () => {
  it('uses a compact vertical detail structure for populated detail', () => {
    const { body } = render(WorkshopDetailPanel, {
      props: {
        detail: {
          id: 'workshop-1',
          title: 'Workshop Forest',
          itemType: 'scene',
          coverPath: null,
          syncStatus: 'synced',
          compatibility: {
            badge: 'fully_supported',
            reasonCode: 'ready_for_library',
            summaryCopy: 'Ready to use',
            headline: 'Ready to use',
            detail: 'This Workshop item is synchronized locally and ready to use.',
            nextStep: 'none',
            nextStepCopy: null
          },
          description: 'Dense detail flow.',
          tags: ['forest']
        },
        loading: false,
        error: null,
        openInSteam: async () => {}
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
    const { body } = render(WorkshopDetailPanel, {
      props: {
        detail: null,
        loading: false,
        error: null,
        openInSteam: null
      }
    });

    expect(body).toContain('Select a Workshop item to inspect its current detail payload.');
    expect(body).toContain('lwe-subpanel');
    expect(body).toContain('border-dashed');
    expect(body).toContain('role="status"');
  });

  it('uses the shared subpanel treatment for the error detail state', () => {
    const { body } = render(WorkshopDetailPanel, {
      props: {
        detail: null,
        loading: false,
        error: 'Unable to complete the Workshop request.',
        openInSteam: null
      }
    });

    expect(body).toContain('Unable to complete the Workshop request.');
    expect(body).toContain('lwe-subpanel');
    expect(body).toContain('lwe-warning-banner lwe-wrap-safe');
    expect(body).toContain('Workshop detail');
  });
});
