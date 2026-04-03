import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import { resetPreferredLanguage, setPreferredLanguage } from '$lib/i18n';
import ItemCard from './ItemCard.svelte';

describe('ItemCard', () => {
  afterEach(() => {
    resetPreferredLanguage();
  });

  it('renders the assembled compatibility summary copy', () => {
    const { body } = render(ItemCard, {
      props: {
        title: 'Forest Scene',
        itemType: 'scene',
        coverPath: null,
        compatibility: {
          badge: 'unsupported',
          reasonCode: 'unsupported_web_item',
          summaryCopy: 'Web support coming later'
        },
        selected: false,
        assignedMonitorLabels: [],
        onApplyShortcut: () => {}
      }
    });

    expect(body).toContain('Web support coming later');
    expect(body).not.toContain('unsupported_web_item');
    expect(body).toContain('data-slot="card"');
    expect(body).toContain('data-slot="badge"');
  });

  it('renders assigned monitor labels from the assembled library quick status', () => {
    const { body } = render(ItemCard, {
      props: {
        title: 'Forest Scene',
        itemType: 'scene',
        coverPath: null,
        compatibility: {
          badge: 'fully_supported',
          reasonCode: 'ready_for_library',
          summaryCopy: 'Ready to use'
        },
        selected: true,
        assignedMonitorLabels: ['Primary', 'DISPLAY-2 (missing)']
      }
    });

    expect(body).toContain('Assigned to');
    expect(body).toContain('Primary');
    expect(body).toContain('DISPLAY-2 (missing)');
  });

  it('exposes a secondary action path on the item surface', () => {
    const { body } = render(ItemCard, {
      props: {
        title: 'Forest Scene',
        itemType: 'scene',
        coverPath: null,
        compatibility: {
          badge: 'fully_supported',
          reasonCode: 'ready_for_library',
          summaryCopy: 'Ready to use'
        },
        selected: false,
        assignedMonitorLabels: [],
        onApplyShortcut: () => {}
      }
    });

    expect(body).toContain('Show quick actions for Forest Scene');
    expect(body).toContain('aria-expanded="false"');
    expect(body).not.toContain('role="menu"');
  });

  it('localizes the item type label from centralized i18n copy', () => {
    setPreferredLanguage('zh-CN');

    const { body } = render(ItemCard, {
      props: {
        title: 'Forest Scene',
        itemType: 'scene',
        coverPath: null,
        compatibility: {
          badge: 'fully_supported',
          reasonCode: 'ready_for_library',
          summaryCopy: 'Ready to use'
        },
        selected: false,
        assignedMonitorLabels: []
      }
    });

    expect(body).toContain('场景');
    expect(body).toContain('完全支持');
    expect(body).not.toContain('Fully Supported');
    expect(body).not.toContain('>scene<');
  });
});
