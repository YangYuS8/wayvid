import { describe, expect, it } from 'vitest';

import { resolveWorkshopRefreshDetailState } from './page-state';

describe('workshop refresh detail state', () => {
  it('resets detail loading and invalidates the in-flight request when refresh clears selection', () => {
    expect(
      resolveWorkshopRefreshDetailState({
        previousSelection: 'item-a',
        nextSelection: null,
        detailLoading: true,
        detailRequestToken: 4,
        detailError: 'Detail request failed'
      })
    ).toEqual({
      detailLoading: false,
      detailRequestToken: 5,
      detailError: null
    });
  });

  it('keeps the current request state when refresh preserves selection', () => {
    expect(
      resolveWorkshopRefreshDetailState({
        previousSelection: 'item-a',
        nextSelection: 'item-a',
        detailLoading: true,
        detailRequestToken: 4,
        detailError: 'Detail request failed'
      })
    ).toEqual({
      detailLoading: true,
      detailRequestToken: 4,
      detailError: 'Detail request failed'
    });
  });
});
