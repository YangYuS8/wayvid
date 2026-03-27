export const shouldRenderCoverImage = (coverPath: string | null, loadFailed: boolean) =>
  Boolean(coverPath) && !loadFailed;
