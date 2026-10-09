import { describe, expect, it } from "vitest";
import { fixtureFor } from "./performer-fixtures";
import {
  keyIntent,
  performerPhase,
  popoverX,
  progressPercent,
  seekCommand,
  seekTarget,
  usageLevel,
  formatLongTime,
  formatTime,
  isWaitingAtHardStop,
} from "./performer";
import screenSource from "../components/PerformerScreen.svelte?raw";
import tokens from "../tokens.css?raw";

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

const song = fixtureFor("Playing").song!;

describe("click to seek (v1 rules)", () => {
  it("maps a click to a position along the bar", () => {
    expect(seekTarget(400, 800, song)).toEqual({ position: 107.5, onCue: false });
    expect(seekTarget(-20, 800, song)).toEqual({ position: 0, onCue: false });
    expect(seekTarget(900, 800, song)).toEqual({ position: 215, onCue: false });
  });

  it("snaps to a cue within 10 px and not beyond", () => {
    const chorusPx = (64 / 215) * 800;
    expect(seekTarget(chorusPx + 10, 800, song)).toEqual({ position: 64, onCue: true });
    expect(seekTarget(chorusPx - 10, 800, song)).toEqual({ position: 64, onCue: true });
    expect(seekTarget(chorusPx + 11, 800, song).onCue).toBe(false);
  });

  it("does not divide by zero on an empty bar or song", () => {
    expect(seekTarget(5, 0, song)).toEqual({ position: 0, onCue: false });
    expect(seekTarget(5, 800, { ...song, duration: 0 })).toEqual({ position: 0, onCue: false });
  });

  it("counts in only for a cue click with the setting on", () => {
    const cue = { position: 64, onCue: true };
    const free = { position: 70, onCue: false };
    expect(seekCommand(cue, true)).toEqual({ Seek: { position: 64, count_in: true } });
    expect(seekCommand(cue, false)).toEqual({ Seek: { position: 64, count_in: false } });
    expect(seekCommand(free, true)).toEqual({ Seek: { position: 70, count_in: false } });
  });

  it("keeps the time popover inside the bar", () => {
    expect(popoverX(5, 800)).toBe(30);
    expect(popoverX(795, 800)).toBe(770);
    expect(popoverX(400, 800)).toBe(400);
    expect(popoverX(5, 40)).toBe(30);
  });
});

describe("keyboard controls", () => {
  it("knows the v1 keys that exist so far", () => {
    expect(keyIntent(" ")).toBe("PlayPause");
    expect(keyIntent("a")).toBe("ToggleAutoResume");
    expect(keyIntent("ArrowLeft")).toBe("Previous");
    expect(keyIntent("ArrowRight")).toBe("Next");
    expect(keyIntent("ArrowUp")).toBeNull();
    expect(keyIntent("A")).toBeNull();
  });
});

describe("system stats", () => {
  it("uses v1 thresholds for the usage colour", () => {
    expect(usageLevel(49)).toBe("low");
    expect(usageLevel(50)).toBe("medium");
    expect(usageLevel(79)).toBe("medium");
    expect(usageLevel(80)).toBe("high");
  });
});

describe("hard-stop flash (v1: 2 s ease-in-out, #121212 to #2a0000, off for reduced motion)", () => {
  it("keeps v1's timing and colours", () => {
    expect(screenSource).toMatch(/animation:\s*flash 2s ease-in-out infinite/);
    expect(screenSource).toMatch(/background:\s*var\(--stage\)/);
    expect(screenSource).toMatch(/50%\s*\{\s*background:\s*var\(--alarm\)/);
    expect(tokens).toMatch(/--stage:\s*#121212/);
    expect(tokens).toMatch(/--alarm:\s*#2a0000/);
    expect(screenSource).toMatch(/prefers-reduced-motion: reduce\)\s*\{\s*\.flash\s*\{\s*animation:\s*none/);
  });

  it("flashes only while waiting at a hard stop with a next song", () => {
    expect(isWaitingAtHardStop(fixtureFor("HardStopped"))).toBe(true);
    expect(isWaitingAtHardStop(fixtureFor("Paused"))).toBe(false);
  });
});

describe("performerPhase", () => {
  it("keeps the phases the screen knows and folds the two it does not", () => {
    expect(performerPhase("CountingIn")).toBe("CountingIn");
    expect(performerPhase("HardStopped")).toBe("HardStopped");
    expect(performerPhase("HandingOver")).toBe("Playing");
    expect(performerPhase("Finished")).toBe("Idle");
  });
});
