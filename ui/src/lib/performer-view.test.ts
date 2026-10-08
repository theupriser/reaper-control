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

const songs = [song("A", 0, 100), song("B", 100, 190, { length: 80, hard_stop: true })];

// `position` is seconds into the current song; the live state carries the timeline position.
const link = (current: number | null, position = 0): LinkView => ({
  status: { Connected: { extension_version: "1" } },
  live: {
    sequence: 1,
    timestamp: 0,
    transport: "Playing",
    position: (current === null ? 0 : songs[current].start) + position,
    phase: "Playing",
    setlist_id: null,
    current_song: current,
    next_song: current !== null && current + 1 < songs.length ? current + 1 : null,
    autoplay: false,
    count_in: true,
    record_armed: false,
    catalog_revision: 1,
    setlist_revision: 0,
  },
  catalog: {
    revision: 1,
    setlist_revision: 0,
    songs,
    project_songs: songs,
    cues: [
      { id: "cue-0", name: "Chorus", position: 130 },
      { id: "cue-1", name: "Outro", position: 50 },
    ],
    setlists: [],
    active_setlist: null,
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
    expect(performerView(link(null), "The extension turned itself off").stats.connected).toBe(false);
  });

  it("names the played setlist and shows nothing for an unknown one", () => {
    const view = link(0);
    view.catalog.setlists = [{ id: "friday", name: "Friday", revision: 1, entries: [] }];
    expect(performerView(view).setlistName).toBeNull();
    view.catalog.active_setlist = "friday";
    expect(performerView(view).setlistName).toBe("Friday");
    view.catalog.active_setlist = "gone";
    expect(performerView(view).setlistName).toBeNull();
  });
});
