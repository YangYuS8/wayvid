import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import WorkshopPage from './+page.svelte';
import { resetPreferredLanguage, setPreferredLanguage } from '$lib/i18n';
import { pageCache } from '$lib/stores/ui';

const resetCache = () => {
  pageCache.set({
    library: { snapshot: null, detail: null, stale: false },
    workshop: { snapshot: null, detail: null, stale: false },
    desktop: { snapshot: null, detail: null, stale: false },
    settings: { snapshot: null, detail: null, stale: false }
  });
};

describe('workshop page render', () => {
  afterEach(() => {
    resetPreferredLanguage();
    resetCache();
  });

  it('does not render the split body when no workshop snapshot is available', () => {
    const { body } = render(WorkshopPage);

    expect(body).toContain('Local Workshop sync');
    expect(body).toContain('Review the current Steam Workshop items synced into Wayvid from this machine. This is not a full online Workshop browser.');
    expect(body).not.toContain('No Workshop items are available in the current snapshot.');
    expect(body).not.toContain('Select a Workshop item to inspect its current detail payload.');
  });

  it('renders route and detail placeholder copy in Simplified Chinese when zh-CN is active', () => {
    setPreferredLanguage('zh-CN');

    pageCache.update((cache) => ({
      ...cache,
      workshop: {
        ...cache.workshop,
        snapshot: {
          items: [],
          selectedItemId: null,
          stale: false
        }
      }
    }));

    const { body } = render(WorkshopPage);

    expect(body).toContain('创意工坊');
    expect(body).toContain('本地创意工坊同步');
    expect(body).toContain('刷新目录');
    expect(body).toContain('当前快照中没有可用的创意工坊项目。');
    expect(body).toContain('工坊详情');
    expect(body).toContain('选择一个创意工坊项目以查看当前详情。');
  });
});
