import { afterEach, describe, expect, it, vi } from 'vitest';

const { invoke } = vi.hoisted(() => ({
  invoke: vi.fn().mockResolvedValue({ ok: false, message: 'unavailable' })
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke
}));

import { applyLibraryItemToMonitor, clearLibraryItemFromMonitor } from './ipc';

describe('ipc desktop flow bridge', () => {
  afterEach(() => {
    invoke.mockClear();
  });

  it('invokes the desktop apply and clear commands with the active flow names', async () => {
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
