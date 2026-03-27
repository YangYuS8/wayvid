type WorkshopRefreshDetailStateInput = {
  previousSelection: string | null;
  nextSelection: string | null;
  detailLoading: boolean;
  detailRequestToken: number;
  detailError: string | null;
};

type WorkshopRefreshDetailState = {
  detailLoading: boolean;
  detailRequestToken: number;
  detailError: string | null;
};

export const resolveWorkshopRefreshDetailState = ({
  previousSelection,
  nextSelection,
  detailLoading,
  detailRequestToken,
  detailError
}: WorkshopRefreshDetailStateInput): WorkshopRefreshDetailState => {
  if (previousSelection && previousSelection !== nextSelection) {
    return {
      detailLoading: false,
      detailRequestToken: detailRequestToken + 1,
      detailError: null
    };
  }

  return {
    detailLoading,
    detailRequestToken,
    detailError
  };
};
