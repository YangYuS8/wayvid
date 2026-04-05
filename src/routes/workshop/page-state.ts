type WorkshopRefreshStateInput = {
  currentSelection: string | null;
  hasCurrentUpdate: boolean;
  availableItemIds: string[];
  detailLoading: boolean;
  detailRequestToken: number;
  detailError: string | null;
};

type WorkshopOnlineSearchTokenInput = {
  requestToken: number;
  responseToken: number;
};

type WorkshopOnlineSearchPageInput = {
  currentPage: number;
  hasMore: boolean;
};

type WorkshopRefreshState = {
  nextSelection: string | null;
  detailLoading: boolean;
  detailRequestToken: number;
  detailError: string | null;
};

export const resolveWorkshopRefreshState = ({
  currentSelection,
  hasCurrentUpdate,
  availableItemIds,
  detailLoading,
  detailRequestToken,
  detailError
}: WorkshopRefreshStateInput): WorkshopRefreshState => {
  if (!hasCurrentUpdate) {
    return {
      nextSelection: null,
      detailLoading: false,
      detailRequestToken: detailRequestToken + 1,
      detailError: null
    };
  }

  const nextSelection =
    currentSelection && availableItemIds.includes(currentSelection) ? currentSelection : null;

  if (currentSelection !== nextSelection) {
    return {
      nextSelection,
      detailLoading: false,
      detailRequestToken: detailRequestToken + 1,
      detailError: null
    };
  }

  return {
    nextSelection,
    detailLoading,
    detailRequestToken,
    detailError
  };
};

export const isLatestWorkshopOnlineSearchResponse = ({
  requestToken,
  responseToken
}: WorkshopOnlineSearchTokenInput) => requestToken === responseToken;

export const nextWorkshopOnlineSearchPage = ({
  currentPage,
  hasMore
}: WorkshopOnlineSearchPageInput) => (hasMore ? currentPage + 1 : currentPage);
