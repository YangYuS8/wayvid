import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import ItemCard from './ItemCard.svelte';

describe('ItemCard', () => {
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
        selected: false
      }
    });

    expect(body).toContain('Web support coming later');
    expect(body).not.toContain('unsupported_web_item');
  });
});
