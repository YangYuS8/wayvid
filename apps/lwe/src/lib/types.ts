export type InvalidatedPage = 'library' | 'workshop' | 'desktop' | 'settings';

export interface AppShellSnapshot {
  appName: string;
  codeName: string;
  steamAvailable: boolean;
  libraryCount: number;
  workshopSyncedCount: number;
  monitorCount: number;
}

export interface WorkshopItemSummary {
  id: number;
  title: string;
  itemType: string;
  coverPath: string | null;
  syncStatus: string;
  compatibilityBadge: string;
}

export interface WorkshopPageSnapshot {
  items: WorkshopItemSummary[];
  selectedItemId: number | null;
  stale: boolean;
}

export interface WorkshopItemDetail {
  id: number;
  title: string;
  itemType: string;
  coverPath: string | null;
  syncStatus: string;
  compatibilityBadge: string;
  compatibilityNote: string | null;
  tags: string[];
  description: string | null;
}

export interface LibraryItemSummary {
  id: string;
  title: string;
  itemType: string;
  coverPath: string | null;
  source: string;
  favorite: boolean;
}

export interface LibraryPageSnapshot {
  items: LibraryItemSummary[];
  selectedItemId: string | null;
  stale: boolean;
}

export interface LibraryItemDetail {
  id: string;
  title: string;
  itemType: string;
  coverPath: string | null;
  source: string;
  description: string | null;
  tags: string[];
}

export interface DesktopMonitorSummary {
  monitorId: string;
  displayName: string;
  resolution: string;
  currentWallpaperTitle: string | null;
  currentCoverPath: string | null;
  runtimeStatus: string;
}

export interface DesktopPageSnapshot {
  monitors: DesktopMonitorSummary[];
  stale: boolean;
}

export interface SettingsPageSnapshot {
  language: string;
  theme: string;
  steamRequired: boolean;
  stale: boolean;
}

export interface AppShellPatch {
  workshopSyncedCount?: number;
  libraryCount?: number;
  monitorCount?: number;
}

export interface ActionOutcome<T> {
  ok: boolean;
  message: string | null;
  shellPatch: AppShellPatch | null;
  currentUpdate: T | null;
  invalidations: InvalidatedPage[];
}
