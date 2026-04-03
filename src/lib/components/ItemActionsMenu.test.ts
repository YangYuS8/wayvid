import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import { resetPreferredLanguage, setPreferredLanguage } from '$lib/i18n';

import ItemActionsMenu from './ItemActionsMenu.svelte';

describe('ItemActionsMenu', () => {
  afterEach(() => {
    resetPreferredLanguage();
  });

  it('renders the localized trigger label in Simplified Chinese', () => {
    setPreferredLanguage('zh-CN');

    const { body } = render(ItemActionsMenu, {
      props: {
        itemTitle: 'Forest Scene',
        onApplyShortcut: () => {}
      }
    });

    expect(body).toContain('aria-label="显示 Forest Scene 的快捷操作"');
  });
});
