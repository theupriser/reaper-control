import { describe, expect, it } from "vitest";
import { UNLOCK_WINDOW, isLocked, lock, open, settle, tapUnlock } from "./stage-lock";

describe("stage lock", () => {
  it("one tap only arms the unlock", () => {
    const armed = tapUnlock(lock(), 1000);
    expect(armed).toEqual({ kind: "unlocking", until: 1000 + UNLOCK_WINDOW });
    expect(isLocked(armed)).toBe(true);
  });

  it("a second tap inside the window opens it", () => {
    expect(tapUnlock(tapUnlock(lock(), 1000), 1000 + UNLOCK_WINDOW)).toEqual(open);
  });

  it("a second tap after the window arms again instead of opening", () => {
    const again = tapUnlock(tapUnlock(lock(), 1000), 1001 + UNLOCK_WINDOW);
    expect(again).toEqual({ kind: "unlocking", until: 1001 + 2 * UNLOCK_WINDOW });
  });

  it("an armed unlock that ran out is locked again", () => {
    const armed = tapUnlock(lock(), 0);
    expect(settle(armed, UNLOCK_WINDOW)).toEqual(armed);
    expect(settle(armed, UNLOCK_WINDOW + 1)).toEqual(lock());
  });

  it("an open screen stays open", () => {
    expect(tapUnlock(open, 5)).toEqual(open);
    expect(isLocked(open)).toBe(false);
  });
});
