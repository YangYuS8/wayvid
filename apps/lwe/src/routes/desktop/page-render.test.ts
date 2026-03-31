import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import DesktopPage from './+page.svelte';
import { pageCache, setDesktopSnapshot } from '$lib/stores/ui';

const resetCache = () => {
  pageCache.set({
    library: { snapshot: null, detail: null, stale: false },
    workshop: { snapshot: null, detail: null, stale: false },
    desktop: { snapshot: null, detail: null, stale: false },
    settings: { snapshot: null, detail: null, stale: false }
  });
};

describe('desktop page render', () => {
  afterEach(() => {
    resetCache();
  });

  it('renders monitor entries and missing monitor restores from the snapshot', () => {
    setDesktopSnapshot({
      monitors: [
        {
          monitorId: 'DISPLAY-1',
          displayName: 'Primary',
          resolution: '1920x1080',
          currentWallpaperTitle: 'Forest Scene',
          currentCoverPath: null,
          currentItemId: 'scene-7',
          restoreState: 'restored',
          restoreIssue: null,
          runtimeStatus: 'unsupported'
        }
      ],
      missingMonitorRestores: [
        {
          monitorId: 'DISPLAY-2',
          currentItemId: 'scene-8',
          currentWallpaperTitle: 'Ocean Scene',
          restoreState: 'missing_monitor',
          restoreIssue: 'Saved assignment targets a monitor that is not currently available.'
        }
      ],
      monitorsAvailable: true,
      monitorDiscoveryIssue: null,
      persistenceIssue: null,
      assignmentsAvailable: true,
      restoreIssues: ['Saved assignment for missing monitor DISPLAY-3 still points to Forest Scene (scene-7).'],
      stale: false
    });

    const { body } = render(DesktopPage);

    expect(body).toContain('Primary');
    expect(body).toContain('1920x1080');
    expect(body).toContain('Forest Scene');
    expect(body).toContain('Missing monitor restores');
    expect(body).toContain('DISPLAY-2');
    expect(body).toContain('Ocean Scene');
    expect(body).toContain(
      'Saved assignment for missing monitor DISPLAY-3 still points to Forest Scene (scene-7).'
    );
  });
});
