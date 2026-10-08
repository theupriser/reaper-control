import { describe, expect, it } from "vitest";
import type { LinkView, SongInfo } from "./generated/protocol";
import { performerView } from "./performer-view";

const song = (id: string, start: number, end: number, extra: Partial<SongInfo> = {}): SongInfo => ({
  id,
  name: `Song ${id}`,
  start,
  end,
  colour: null,
  hard_stop: false,
  length: null,
  bpm: null,
  ...extra,
});

const link = (current: number | null, position = 0): LinkView => ({
  status: { Connected: { extension_version: "1" } },
  state: {
    phase: "Playing",
    position,
    auto_resume: false,
    count_in_on_marker: true,
    record_armed: false,
    current_song: current,
  },
  catalog: {
    revision: 1,
    setlist_revision: 0,
    songs: [song("A", 0, 100), song("B", 100, 190, { length: 80, hard_stop: true })],
    cues: [
      { id: "cue-0", name: "Chorus", position: 130 },
      { id: "cue-1", name: "Outro", position: 50 },
    ],
    setlists: [],
  },
});

describe("performerView", () => {
  it("shows the current and the next song with real lengths", () => {
    const view = performerView(link(0, 12));
    expect(view.song?.name).toBe("Song A");
    expect(view.song?.duration).toBe(100);
    expect(view.nextSong).toEqual({ name: "Song B", duration: 80 });
    expect(view.totalDuration).toBe(180);
    expect(view.totalElapsed).toBe(12);
  });

  it("counts the finished songs into the total and keeps only this song's cues, relative to its start", () => {
    const view = performerView(link(1, 10));
    expect(view.totalElapsed).toBe(110);
    expect(view.nextSong).toBeNull();
    expect(view.song?.hardStop).toBe(true);
    expect(view.song?.cues).toEqual([{ name: "Chorus", position: 30 }]);
  });

  it("allows Previous only after the first song", () => {
    expect(performerView(link(0)).hasPrevious).toBe(false);
    expect(performerView(link(1)).hasPrevious).toBe(true);
    expect(performerView(link(null)).hasPrevious).toBe(false);
  });

  it("shows no song while the extension reports none or is not running", () => {
    expect(performerView(link(null)).song).toBeNull();
    const down: LinkView = { ...link(0), status: "NotRunning" };
    expect(performerView(down).stats.connected).toBe(false);
  });
});
