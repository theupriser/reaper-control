import { describe, expect, it } from "vitest";
import { fixtureFor } from "./performer-fixtures";
import { formatLongTime, formatTime, isWaitingAtHardStop, progressPercent } from "./performer";

describe("performer formatting", () => {
  it("formats song and setlist times", () => {
    expect(formatTime(0)).toBe("0:00");
    expect(formatTime(185.9)).toBe("3:05");
    expect(formatTime(-4)).toBe("0:00");
    expect(formatLongTime(3725)).toBe("1:02:05");
  });

  it("clamps progress to 0..100", () => {
    expect(progressPercent(50, 200)).toBe(25);
    expect(progressPercent(300, 200)).toBe(100);
    expect(progressPercent(-1, 200)).toBe(0);
    expect(progressPercent(10, 0)).toBe(0);
  });

  it("waits at a hard stop only when a next song exists", () => {
    const stopped = fixtureFor("HardStopped");
    expect(isWaitingAtHardStop(stopped)).toBe(true);
    expect(isWaitingAtHardStop({ ...stopped, nextSong: null })).toBe(false);
    expect(isWaitingAtHardStop(fixtureFor("Playing"))).toBe(false);
  });
});
