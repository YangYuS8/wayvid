import { convertFileSrc } from '@tauri-apps/api/core';

export const shouldRenderCoverImage = (coverPath: string | null, loadFailed: boolean) =>
  Boolean(coverPath) && !loadFailed;

export const resolveCoverSrc = (coverPath: string | null) => {
  if (!coverPath) {
    return undefined;
  }

  if (/^(https?:|asset:|tauri:|file:|blob:|data:)/.test(coverPath)) {
    return coverPath;
  }

  if (coverPath.startsWith('/')) {
    if (typeof window === 'undefined') {
      return undefined;
    }

    return convertFileSrc(coverPath);
  }

  return coverPath;
};
