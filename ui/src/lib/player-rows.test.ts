import { describe, expect, it } from "vitest";
import type { Catalog, Live } from "./generated/protocol";
import { playerRows } from "./player-rows";

const song = (number: number, extra = {}) => ({ id: `s${number}`, number, name: `Song ${number}`, start: number * 100, end: number * 100 + 90, colour: null, hard_stop: false, length: null, bpm: null, ...extra });
const catalog = { songs: [song(1), song(2, { bpm: 128, hard_stop: true }), song(3, { length: 30 }), song(4)] } as unknown as Catalog;
const live = (current: number | null, next: number | null) => ({ current_song: current, next_song: next }) as Live;

describe("playerRows", () => {
  it("marks played, current, next and upcoming songs", () => {
    expect(playerRows(catalog, live(1, 2)).map((row) => row.state)).toEqual(["played", "current", "next", "upcoming"]);
  });

  it("marks nothing as played before the show starts", () => {
    expect(playerRows(catalog, live(null, null)).map((row) => row.state)).toEqual(["upcoming", "upcoming", "upcoming", "upcoming"]);
    expect(playerRows(catalog, null)).toHaveLength(4);
  });

  it("carries number, length, tempo and hard stop", () => {
    const rows = playerRows(catalog, live(0, 1));
    expect(rows[1]).toMatchObject({ number: 2, bpm: 128, hardStop: true, duration: 90 });
    expect(rows[2].duration).toBe(30);
  });
});

describe("playerRows memory", () => {
  const stable = { revision: 1, setlist_revision: 1, active_setlist: "set", songs: [song(1), song(2)] } as unknown as Catalog;

  it("gives the same rows back when only the position moved", () => {
    expect(playerRows(stable, live(0, 1))).toBe(playerRows({ ...stable }, live(0, 1)));
  });

  it("builds new rows when the current song or the catalog changes", () => {
    const first = playerRows(stable, live(0, 1));
    expect(playerRows(stable, live(1, null))).not.toBe(first);
    expect(playerRows({ ...stable, revision: 2 }, live(1, null))).not.toBe(first);
  });
});
