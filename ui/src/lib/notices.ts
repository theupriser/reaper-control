import type { Notice } from "./generated/protocol";

export type ShownNotice = Notice & { shownAt: number };

/** How long a notice stays: info 2 s, warning 10 s, an error until it is dismissed (canvas, States). */
const lifetimeMilliseconds = { Info: 2000, Warning: 10000, Error: Infinity } as const;

/** A newer notice replaces the older one with the same key. */
export const addNotice = (shown: ShownNotice[], notice: Notice, now: number): ShownNotice[] => [
  ...shown.filter((other) => other.key !== notice.key),
  { ...notice, shownAt: now },
];

/** Drops the notices that have been on screen long enough for their level. */
export const expireNotices = (shown: ShownNotice[], now: number): ShownNotice[] =>
  shown.filter((notice) => now - notice.shownAt < lifetimeMilliseconds[notice.level]);

/** Removes the notice a person dismissed. */
export const dismissNotice = (shown: ShownNotice[], key: string): ShownNotice[] =>
  shown.filter((notice) => notice.key !== key);
