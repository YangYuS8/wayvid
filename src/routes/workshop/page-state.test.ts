import { describe, expect, it } from 'vitest';

import {
  isLatestWorkshopOnlineSearchResponse,
  nextWorkshopOnlineSearchPage,
  resolveWorkshopRefreshState
} from './page-state';

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

describe('workshop online search token guard', () => {
  it('accepts only the latest response token', () => {
    expect(
      isLatestWorkshopOnlineSearchResponse({
        requestToken: 4,
        responseToken: 4
      })
    ).toBe(true);

    expect(
      isLatestWorkshopOnlineSearchResponse({
        requestToken: 3,
        responseToken: 4
      })
    ).toBe(false);
  });
});

describe('workshop online pagination helper', () => {
  it('increments page when more results are available', () => {
    expect(nextWorkshopOnlineSearchPage({ currentPage: 2, hasMore: true })).toBe(3);
  });

  it('keeps page unchanged when no more results are available', () => {
    expect(nextWorkshopOnlineSearchPage({ currentPage: 2, hasMore: false })).toBe(2);
  });
});
