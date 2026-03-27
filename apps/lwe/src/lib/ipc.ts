import type {
  ActionOutcome,
  AppShellSnapshot,
  DesktopPageSnapshot,
  LibraryItemDetail,
  LibraryPageSnapshot,
  SettingsPageSnapshot,
  WorkshopItemDetail,
  WorkshopPageSnapshot
} from '$lib/types';

type InvokeArgs = Record<string, unknown>;

type TauriBridge = {
  invoke?: <T>(command: string, args?: InvokeArgs) => Promise<T>;
};

declare global {
  interface Window {
    __TAURI__?: {
      core?: TauriBridge;
    };
  }
}

const invokeCommand = <T>(command: string, args?: InvokeArgs) => {
  const invoke = window.__TAURI__?.core?.invoke;

  if (!invoke) {
    return Promise.reject(new Error(`Tauri bridge is unavailable for ${command}`));
  }

  return invoke<T>(command, args);
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

export const loadDesktopPage = () => invokeCommand<DesktopPageSnapshot>('load_desktop_page');

export const loadSettingsPage = () => invokeCommand<SettingsPageSnapshot>('load_settings_page');
