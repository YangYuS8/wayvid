import type { InvalidatedPage } from '$lib/types';
import { applyInvalidations } from '$lib/stores/ui';

export const applyDesktopClearInvalidations = (invalidations: InvalidatedPage[]) => {
  applyInvalidations(invalidations);
};
