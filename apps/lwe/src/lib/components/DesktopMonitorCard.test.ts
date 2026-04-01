import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import DesktopMonitorCard from './DesktopMonitorCard.svelte';

describe('DesktopMonitorCard', () => {
  it('renders media and restore state through the shared product card structure', () => {
    const { body } = render(DesktopMonitorCard, {
      props: {
        displayName: 'Primary',
        monitorId: 'DISPLAY-1',
        resolution: '1920x1080',
        currentItemLabel: 'Forest Scene',
        currentCoverPath: '/covers/forest-scene.jpg',
        runtimeStatus: 'unsupported',
        restoreState: 'restored',
        restoreIssue: 'Saved assignment was restored from the last session.',
        missing: false
      }
    });

    expect(body).toContain('Primary');
    expect(body).toContain('DISPLAY-1');
    expect(body).toContain('1920x1080');
    expect(body).toContain('Forest Scene');
    expect(body).toContain('Unsupported');
    expect(body).toContain('Restored');
    expect(body).toContain('/covers/forest-scene.jpg');
    expect(body).toContain('Primary current item');
    expect(body).toContain('Restore state');
    expect(body).toContain('Saved assignment was restored from the last session.');
    expect(body).toContain('data-slot="card"');
    expect(body).toContain('View status details');
    expect(body).toContain('aria-expanded="false"');
  });
});
