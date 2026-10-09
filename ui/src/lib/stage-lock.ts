/** How long the second tap to unlock stays valid, in milliseconds. */
export const UNLOCK_WINDOW = 3000;

export type StageLock = { kind: "open" } | { kind: "locked" } | { kind: "unlocking"; until: number };

export const open: StageLock = { kind: "open" };

export const lock = (): StageLock => ({ kind: "locked" });

/** The first tap on Unlock only arms it; a second tap within the window opens. */
export function tapUnlock(state: StageLock, now: number): StageLock {
  if (state.kind === "open") return state;
  if (state.kind === "unlocking" && now <= state.until) return open;
  return { kind: "unlocking", until: now + UNLOCK_WINDOW };
}

/** An armed unlock that ran out goes back to locked. */
export function settle(state: StageLock, now: number): StageLock {
  return state.kind === "unlocking" && now > state.until ? lock() : state;
}

export const isLocked = (state: StageLock): boolean => state.kind !== "open";
