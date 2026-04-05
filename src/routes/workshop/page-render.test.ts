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
    expect(body).toContain('Review local sync state and search Steam Workshop online with saved filters.');
    expect(body).toContain('Online search');
    expect(body).not.toContain('No Workshop items are available in the current snapshot.');
    expect(body).not.toContain('Select a Workshop item to inspect its current detail payload.');
  });

  it('renders route and detail placeholder copy in Simplified Chinese when zh-CN is active', () => {
    setPreferredLanguage('zh-CN');

    const { body } = render(WorkshopPage);

    expect(body).toContain('创意工坊');
    expect(body).toContain('本地创意工坊同步');
    expect(body).toContain('在线搜索');
    expect(body).toContain('显示筛选');
    expect(body).not.toContain('当前快照中没有可用的创意工坊项目。');
    expect(body).not.toContain('工坊详情');
  });
});
