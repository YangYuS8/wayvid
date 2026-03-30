export type InvalidatedPage = 'library' | 'workshop' | 'desktop' | 'settings';
export type ItemType = 'video' | 'scene' | 'web' | 'other';
export type WorkshopSyncStatus = 'synced' | 'missing_project' | 'missing_asset' | 'unsupported_type';
export type CompatibilityBadge = 'fully_supported' | 'partially_supported' | 'unsupported';
export type CompatibilityNextStep =
  | 'none'
  | 'open_in_steam'
  | 'resync_workshop_item'
  | 'wait_for_future_support';
export type LibrarySource = 'local' | 'workshop' | 'core' | 'other';
export type RuntimeStatus = 'running' | 'idle' | 'unsupported' | 'error';

export interface CompatibilityBaseModel {
  badge: CompatibilityBadge;
  reasonCode: string;
}

export interface CompatibilitySummaryModel extends CompatibilityBaseModel {
  summaryCopy: string;
}

export interface CompatibilityExplanationModel extends CompatibilityBaseModel {
  summaryCopy: string;
  headline: string;
  detail: string;
  nextStep: CompatibilityNextStep;
  nextStepCopy: string | null;
}

export interface AppShellSnapshot {
  appName: string;
  codeName: string;
  steamAvailable: boolean;
  libraryCount: number | null;
  workshopSyncedCount: number | null;
  monitorCount: number | null;
}

export interface WorkshopItemSummary {
  id: string;
  title: string;
  itemType: ItemType;
  coverPath: string | null;
  syncStatus: WorkshopSyncStatus;
  compatibility: CompatibilitySummaryModel;
}

export interface WorkshopPageSnapshot {
  items: WorkshopItemSummary[];
  selectedItemId: string | null;
  stale: boolean;
}

export interface WorkshopItemDetail {
  id: string;
  title: string;
  itemType: ItemType;
  coverPath: string | null;
  syncStatus: WorkshopSyncStatus;
  compatibility: CompatibilityExplanationModel;
  tags: string[];
  description: string | null;
}

export interface LibraryItemSummary {
  id: string;
  title: string;
  itemType: ItemType;
  coverPath: string | null;
  source: LibrarySource;
  compatibility: CompatibilitySummaryModel;
  favorite: boolean;
}

export interface LibraryPageSnapshot {
  items: LibraryItemSummary[];
  selectedItemId: string | null;
  monitorsAvailable: boolean;
  monitorDiscoveryIssue?: string | null;
  desktopAssignmentIssue?: string | null;
  desktopAssignmentsAvailable: boolean;
  stale: boolean;
}

export interface LibraryItemDetail {
  id: string;
  title: string;
  itemType: ItemType;
  coverPath: string | null;
  source: LibrarySource;
  compatibility: CompatibilityExplanationModel;
  monitorsAvailable: boolean;
  monitorDiscoveryIssue?: string | null;
  desktopAssignmentIssue?: string | null;
  desktopAssignmentsAvailable: boolean;
  description: string | null;
  tags: string[];
}

export interface DesktopMonitorSummary {
  monitorId: string;
  displayName: string;
  resolution: string;
  currentWallpaperTitle: string | null;
  currentCoverPath: string | null;
  runtimeStatus: RuntimeStatus;
}

export interface DesktopPageSnapshot {
  monitors: DesktopMonitorSummary[];
  monitorsAvailable: boolean;
  monitorDiscoveryIssue?: string | null;
  persistenceIssue?: string | null;
  assignmentsAvailable: boolean;
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
