type WorkshopRefreshDetailStateInput = {
  previousSelection: string | null;
  nextSelection: string | null;
  detailLoading: boolean;
  detailRequestToken: number;
};

type WorkshopRefreshDetailState = {
  detailLoading: boolean;
  detailRequestToken: number;
};

export const resolveWorkshopRefreshDetailState = ({
  previousSelection,
  nextSelection,
  detailLoading,
  detailRequestToken
}: WorkshopRefreshDetailStateInput): WorkshopRefreshDetailState => {
  if (previousSelection && previousSelection !== nextSelection) {
    return {
      detailLoading: false,
      detailRequestToken: detailRequestToken + 1
    };
  }

  return {
    detailLoading,
    detailRequestToken
  };
};
