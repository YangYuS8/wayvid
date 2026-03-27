type WorkshopRefreshStateInput = {
  previousSelection: string | null;
  currentSelection: string | null;
  availableItemIds: string[];
  detailLoading: boolean;
  detailRequestToken: number;
  detailError: string | null;
};

type WorkshopRefreshState = {
  nextSelection: string | null;
  detailLoading: boolean;
  detailRequestToken: number;
  detailError: string | null;
};

export const resolveWorkshopRefreshState = ({
  previousSelection,
  currentSelection,
  availableItemIds,
  detailLoading,
  detailRequestToken,
  detailError
}: WorkshopRefreshStateInput): WorkshopRefreshState => {
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
