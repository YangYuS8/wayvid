export type DesktopClearState = Set<string>;

export const startDesktopClear = (state: DesktopClearState, monitorId: string): DesktopClearState => {
  const nextState = new Set(state);
  nextState.add(monitorId);
  return nextState;
};

export const finishDesktopClear = (state: DesktopClearState, monitorId: string): DesktopClearState => {
  const nextState = new Set(state);
  nextState.delete(monitorId);
  return nextState;
};

export const isDesktopClearInFlight = (state: DesktopClearState, monitorId: string) => state.has(monitorId);
