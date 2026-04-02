import { describe, expect, it } from 'vitest';

import { resolveWorkshopRefreshState } from './page-state';

describe('workshop refresh state', () => {
  it('preserves a newer in-flight selection during refresh when the item still exists', () => {
    expect(
      resolveWorkshopRefreshState({
        currentSelection: 'item-b',
        hasCurrentUpdate: true,
        availableItemIds: ['item-a', 'item-b'],
        detailLoading: true,
        detailRequestToken: 4,
        detailError: 'Detail request failed'
      })
    ).toEqual({
      nextSelection: 'item-b',
      detailLoading: true,
      detailRequestToken: 4,
      detailError: 'Detail request failed'
    });
  });

  it('clears selection and resets detail state when the latest selection disappears after refresh', () => {
    expect(
      resolveWorkshopRefreshState({
        currentSelection: 'item-b',
        hasCurrentUpdate: true,
        availableItemIds: ['item-a'],
        detailLoading: true,
        detailRequestToken: 4,
        detailError: 'Detail request failed'
      })
    ).toEqual({
      nextSelection: null,
      detailLoading: false,
      detailRequestToken: 5,
      detailError: null
      });
  });

  it('clears selection and detail state when refresh succeeds without a current snapshot update', () => {
    expect(
      resolveWorkshopRefreshState({
        currentSelection: 'item-b',
        hasCurrentUpdate: false,
        availableItemIds: [],
        detailLoading: true,
        detailRequestToken: 4,
        detailError: 'Detail request failed'
      })
    ).toEqual({
      nextSelection: null,
      detailLoading: false,
      detailRequestToken: 5,
      detailError: null
    });
  });
});
