import { get, writable } from 'svelte/store';

import type {
  DesktopPageSnapshot,
  InvalidatedPage,
  LibraryItemDetail,
  LibraryPageSnapshot,
  SettingsPageSnapshot,
  WorkshopAgeRating,
  WorkshopItemDetail,
  WorkshopOnlineItemType,
  WorkshopOnlineSearchResult,
  WorkshopPageSnapshot
} from '$lib/types';

export type PageKey = 'library' | 'workshop' | 'desktop' | 'settings';

type DetailPageKey = 'library' | 'workshop';

type CacheEntry<Snapshot, Detail> = {
  snapshot: Snapshot | null;
  detail: Detail | null;
  stale: boolean;
};

type PageCache = {
  library: CacheEntry<LibraryPageSnapshot, LibraryItemDetail>;
  workshop: CacheEntry<WorkshopPageSnapshot, WorkshopItemDetail>;
  desktop: CacheEntry<DesktopPageSnapshot, null>;
  settings: CacheEntry<SettingsPageSnapshot, null>;
};

const createEmptyCache = (): PageCache => ({
  library: { snapshot: null, detail: null, stale: false },
  workshop: { snapshot: null, detail: null, stale: false },
  desktop: { snapshot: null, detail: null, stale: false },
  settings: { snapshot: null, detail: null, stale: false }
});

export const currentPage = writable<PageKey>('library');
export const pageCache = writable<PageCache>(createEmptyCache());

type WorkshopOnlineCache = {
  query: string;
  ageRatings: WorkshopAgeRating[];
  itemTypes: WorkshopOnlineItemType[];
  pageSize: number;
  result: WorkshopOnlineSearchResult | null;
};

const createEmptyWorkshopOnlineCache = (): WorkshopOnlineCache => ({
  query: '',
  ageRatings: ['g', 'pg_13'],
  itemTypes: ['video', 'scene', 'web', 'application'],
  pageSize: 24,
  result: null
});

export const workshopOnlineCache = writable<WorkshopOnlineCache>(createEmptyWorkshopOnlineCache());

export type ThemePreference = 'light' | 'dark' | 'system';

type ThemeState = {
  preference: ThemePreference;
  effective: 'light' | 'dark';
};

const getSystemTheme = (): 'light' | 'dark' => {
  if (typeof window === 'undefined') {
    return 'light';
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
};

const createThemeState = (preference: ThemePreference): ThemeState => ({
  preference,
  effective: preference === 'system' ? getSystemTheme() : preference
});

export const themeState = writable<ThemeState>(createThemeState('system'));

let systemThemeMediaQuery: MediaQueryList | null = null;
let systemThemeListener: ((event: MediaQueryListEvent) => void) | null = null;

const applyThemeClass = (effective: 'light' | 'dark') => {
  if (typeof document === 'undefined') {
    return;
  }

  document.documentElement.classList.toggle('dark', effective === 'dark');
};

const detachSystemThemeListener = () => {
  if (!systemThemeMediaQuery || !systemThemeListener) {
    return;
  }

  if (typeof systemThemeMediaQuery.removeEventListener === 'function') {
    systemThemeMediaQuery.removeEventListener('change', systemThemeListener);
  } else {
    systemThemeMediaQuery.removeListener(systemThemeListener);
  }

  systemThemeListener = null;
};

const attachSystemThemeListener = () => {
  if (typeof window === 'undefined') {
    return;
  }

  systemThemeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  systemThemeListener = (event: MediaQueryListEvent) => {
    themeState.update((state) => {
      if (state.preference !== 'system') {
        return state;
      }

      const effective = event.matches ? 'dark' : 'light';
      applyThemeClass(effective);
      return { ...state, effective };
    });
  };

  if (typeof systemThemeMediaQuery.addEventListener === 'function') {
    systemThemeMediaQuery.addEventListener('change', systemThemeListener);
  } else {
    systemThemeMediaQuery.addListener(systemThemeListener);
  }
};

export const applyThemePreference = (preference: ThemePreference) => {
  detachSystemThemeListener();
  const nextState = createThemeState(preference);
  applyThemeClass(nextState.effective);
  themeState.set(nextState);

  if (preference === 'system') {
    attachSystemThemeListener();
  }
};

export const setWorkshopOnlineCache = (cache: WorkshopOnlineCache) => {
  workshopOnlineCache.set(cache);
};

export const setCurrentPage = (page: PageKey) => currentPage.set(page);

export const needsPageLoad = (page: PageKey) => {
  const entry = get(pageCache)[page];
  return entry.snapshot === null || entry.stale;
};

export const setLibrarySnapshot = (snapshot: LibraryPageSnapshot) => {
  pageCache.update((cache) => ({
    ...cache,
    library: {
      snapshot,
      detail:
        snapshot.selectedItemId && cache.library.detail?.id === snapshot.selectedItemId
          ? cache.library.detail
          : null,
      stale: false
    }
  }));
};

export const setWorkshopSnapshot = (snapshot: WorkshopPageSnapshot) => {
  pageCache.update((cache) => ({
    ...cache,
    workshop: {
      snapshot,
      detail:
        snapshot.selectedItemId && cache.workshop.detail?.id === snapshot.selectedItemId
          ? cache.workshop.detail
          : null,
      stale: false
    }
  }));
};

export const setDesktopSnapshot = (snapshot: DesktopPageSnapshot) => {
  pageCache.update((cache) => ({
    ...cache,
    desktop: {
      snapshot,
      detail: null,
      stale: false
    }
  }));
};

export const setSettingsSnapshot = (snapshot: SettingsPageSnapshot) => {
  pageCache.update((cache) => ({
    ...cache,
    settings: {
      snapshot,
      detail: null,
      stale: false
    }
  }));
};

export const setPageStale = (page: PageKey, stale = true) => {
  pageCache.update((cache) => ({
    ...cache,
    [page]: {
      ...cache[page],
      stale
    }
  }));
};

export const applyInvalidations = (pages: InvalidatedPage[]) => {
  for (const page of pages) {
    setPageStale(page, true);
  }
};

export const setSelectedItem = (page: DetailPageKey, itemId: string | null) => {
  pageCache.update((cache) => {
    const entry = cache[page];
    if (!entry.snapshot) {
      return cache;
    }

    return {
      ...cache,
      [page]: {
        ...entry,
        snapshot: {
          ...entry.snapshot,
          selectedItemId: itemId
        },
        detail: entry.detail?.id === itemId ? entry.detail : null
      }
    };
  });
};

export const setLibraryDetail = (detail: LibraryItemDetail | null) => {
  pageCache.update((cache) => ({
    ...cache,
    library: {
      ...cache.library,
      detail
    }
  }));
};

export const isSelectedItem = (page: DetailPageKey, itemId: string | null) => {
  const snapshot = get(pageCache)[page].snapshot;
  return snapshot?.selectedItemId === itemId;
};

export const setLibraryDetailIfSelected = (detail: LibraryItemDetail | null, itemId: string | null) => {
  if (!isSelectedItem('library', itemId)) {
    return;
  }

  setLibraryDetail(detail);
};

export const setWorkshopDetail = (detail: WorkshopItemDetail | null) => {
  pageCache.update((cache) => ({
    ...cache,
    workshop: {
      ...cache.workshop,
      detail
    }
  }));
};

export const setWorkshopDetailIfSelected = (detail: WorkshopItemDetail | null, itemId: string | null) => {
  if (!isSelectedItem('workshop', itemId)) {
    return;
  }

  setWorkshopDetail(detail);
};
