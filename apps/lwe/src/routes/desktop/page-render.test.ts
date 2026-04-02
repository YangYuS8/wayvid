import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';
import { get } from 'svelte/store';

import DesktopPage from './+page.svelte';
import { resetPreferredLanguage, setPreferredLanguage } from '$lib/i18n';
import { pageCache, setDesktopSnapshot, setLibrarySnapshot } from '$lib/stores/ui';
import { applyDesktopClearInvalidations } from './page-actions';
import { finishDesktopClear, isDesktopClearInFlight, startDesktopClear } from './clear-state';

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
    resetPreferredLanguage();
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
          clearSupported: true,
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
    expect(body).toContain('Monitor view filter');
    expect(body).toContain('All outputs');
    expect(body).toContain('Current monitors');
    expect(body).toContain('Missing monitor restores');
    expect(body).toContain('DISPLAY-2');
    expect(body).toContain('Ocean Scene');
    expect(body).toContain(
      'Saved assignment for missing monitor DISPLAY-3 still points to Forest Scene (scene-7).'
    );
  });

  it('renders zh-CN route and monitor card copy when Simplified Chinese is active', () => {
    setPreferredLanguage('zh-CN');

    setDesktopSnapshot({
      monitors: [
        {
          monitorId: 'DISPLAY-1',
          displayName: 'Primary',
          resolution: '1920x1080',
          currentWallpaperTitle: null,
          currentCoverPath: null,
          currentItemId: null,
          clearSupported: true,
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

    expect(body).toContain('桌面');
    expect(body).toContain('显示器概览');
    expect(body).toContain('用当前桌面快照查看显示器输出与恢复状态。');
    expect(body).toContain('视图');
    expect(body).toContain('全部输出');
    expect(body).toContain('当前显示器');
    expect(body).toContain('缺失显示器恢复项');
    expect(body).toContain('显示器');
    expect(body).toContain('桌面');
    expect(body).toContain('当前内容');
    expect(body).toContain('没有已保存的分配');
    expect(body).not.toContain('No saved assignment');
    expect(body).not.toContain('>Desktop<');
  });

  it('applies desktop clear invalidations through the shared page cache pattern', () => {
    setLibrarySnapshot({
      items: [],
      selectedItemId: null,
      monitorsAvailable: true,
      desktopAssignmentIssue: null,
      desktopAssignmentsAvailable: true,
      stale: false
    });

    applyDesktopClearInvalidations(['library']);

    const cache = get(pageCache);

    expect(cache.library.stale).toBe(true);
    expect(cache.library.snapshot).not.toBeNull();
    expect(cache.desktop.stale).toBe(false);
  });

  it('tracks overlapping clear actions per monitor instead of dropping the remaining in-flight clear', () => {
    let state = startDesktopClear(new Set<string>(), 'DISPLAY-1');
    state = startDesktopClear(state, 'DISPLAY-2');

    expect(isDesktopClearInFlight(state, 'DISPLAY-1')).toBe(true);
    expect(isDesktopClearInFlight(state, 'DISPLAY-2')).toBe(true);

    state = finishDesktopClear(state, 'DISPLAY-1');

    expect(isDesktopClearInFlight(state, 'DISPLAY-1')).toBe(false);
    expect(isDesktopClearInFlight(state, 'DISPLAY-2')).toBe(true);
  });
});
