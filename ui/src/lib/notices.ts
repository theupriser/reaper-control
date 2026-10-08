import type { Notice } from "./generated/protocol";

export type ShownNotice = Notice & { shownAt: number };

const lifetimeMilliseconds = { Info: 4000, Warning: 8000, Error: 15000 } as const;

/** A newer notice replaces the older one with the same key. */
export const addNotice = (shown: ShownNotice[], notice: Notice, now: number): ShownNotice[] => [
  ...shown.filter((other) => other.key !== notice.key),
  { ...notice, shownAt: now },
];

/** Drops the notices that have been on screen long enough for their level. */
export const expireNotices = (shown: ShownNotice[], now: number): ShownNotice[] =>
  shown.filter((notice) => now - notice.shownAt < lifetimeMilliseconds[notice.level]);
