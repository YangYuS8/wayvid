import { invoke } from '@tauri-apps/api/core';

import type {
  ActionOutcome,
  AppShellSnapshot,
  DesktopPageSnapshot,
  DiagnosticsSnapshot,
  ImportedSceneImage,
  LibraryItemDetail,
  LibraryPageSnapshot,
  ScenePropertiesSnapshot,
  ScenePropertyValue,
  SettingsPageSnapshot,
  SettingsUpdateInput,
  WorkshopItemDetail,
  WorkshopOnlineSearchInput,
  WorkshopOnlineSearchResult,
  WorkshopPageSnapshot
} from '$lib/types';

type InvokeArgs = Record<string, unknown>;

const invokeCommand = <T>(command: string, args?: InvokeArgs) => {
  return invoke<T>(command, args).catch((reason: unknown) => {
    throw typeof reason === 'string' ? new Error(reason) : reason;
  });
};

export const loadAppShell = () => invokeCommand<AppShellSnapshot>('load_app_shell');

export const loadLibraryPage = () => invokeCommand<LibraryPageSnapshot>('load_library_page');

export const loadLibraryItemDetail = (itemId: string) =>
  invokeCommand<LibraryItemDetail>('load_library_item_detail', { itemId });

export const loadWorkshopPage = () => invokeCommand<WorkshopPageSnapshot>('load_workshop_page');

export const loadWorkshopItemDetail = (workshopId: string) =>
  invokeCommand<WorkshopItemDetail>('load_workshop_item_detail', { workshopId });

export const refreshWorkshopCatalog = () =>
  invokeCommand<ActionOutcome<WorkshopPageSnapshot>>('refresh_workshop_catalog');

export const openWorkshopInSteam = (workshopId: string) =>
  invokeCommand<ActionOutcome<null>>('open_workshop_in_steam', { workshopId });

export const searchWorkshopOnline = (input: WorkshopOnlineSearchInput) =>
  invokeCommand<WorkshopOnlineSearchResult>('search_workshop_online', { input });

export const loadDesktopPage = () => invokeCommand<DesktopPageSnapshot>('load_desktop_page');

export const loadDiagnostics = () => invokeCommand<DiagnosticsSnapshot>('load_diagnostics');

export const applyLibraryItemToMonitor = (monitorId: string, itemId: string) =>
  invokeCommand<ActionOutcome<null>>('apply_library_item_to_monitor', { monitorId, itemId });

export const loadSceneProperties = (monitorId: string, itemId: string) =>
  invokeCommand<ScenePropertiesSnapshot>('load_scene_properties', { monitorId, itemId });

export const chooseSceneImage = (itemId: string, propertyName: string) =>
  invokeCommand<ImportedSceneImage | null>('choose_scene_image', { itemId, propertyName });

export const updateSceneProperties = (
  monitorId: string,
  itemId: string,
  values: Record<string, ScenePropertyValue>
) => invokeCommand<ActionOutcome<null>>('update_scene_properties', { monitorId, itemId, values });

export const clearLibraryItemFromMonitor = (monitorId: string) =>
  invokeCommand<ActionOutcome<null>>('clear_library_item_from_monitor', { monitorId });

export const loadSettingsPage = () => invokeCommand<SettingsPageSnapshot>('load_settings_page');

export const updateSettings = (input: SettingsUpdateInput) =>
  invokeCommand<ActionOutcome<SettingsPageSnapshot>>('update_settings', { input });
