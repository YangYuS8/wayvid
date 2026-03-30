import { afterEach, describe, expect, it, vi } from 'vitest';

import { applyLibraryItemToMonitor, clearLibraryItemFromMonitor } from './ipc';

type TestWindow = {
  __TAURI__?: {
    core?: {
      invoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
    };
  };
};

describe('ipc desktop flow bridge', () => {
  afterEach(() => {
    Reflect.deleteProperty(globalThis, 'window');
  });

  it('invokes the desktop apply and clear commands with the active flow names', async () => {
    const invoke = vi.fn().mockResolvedValue({ ok: false, message: 'unavailable' });
    Object.defineProperty(globalThis, 'window', {
      value: { __TAURI__: { core: { invoke } } } as TestWindow,
      configurable: true,
      writable: true
    });

    await applyLibraryItemToMonitor('DISPLAY-1', 'item-1');
    await clearLibraryItemFromMonitor('DISPLAY-1');

    expect(invoke).toHaveBeenNthCalledWith(1, 'apply_library_item_to_monitor', {
      monitorId: 'DISPLAY-1',
      itemId: 'item-1'
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'clear_library_item_from_monitor', {
      monitorId: 'DISPLAY-1'
    });
  });
});
