import { describe, expect, it } from "vitest";
import type { SetlistInfo, SongInfo } from "./generated/protocol";
import { draftOf } from "./setlist-edit";
import { entryRows, matchingSongs, totalDuration } from "./setlist-rows";

const song = (id: string, name: string, start: number, end: number): SongInfo => ({ id, number: 1, name, start, end, colour: null, hard_stop: false, length: null, bpm: null });
const songs = [song("a", "Opener", 0, 100), song("b", "Ballad", 100, 280), song("c", "Closer", 280, 300)];
const saved: SetlistInfo = { id: "sat", name: "Saturday", revision: 3, entries: [{ id: 1, song_id: "a" }, { id: 4, song_id: "gone" }, { id: 5, song_id: "c" }] };

describe("setlist rows", () => {
  it("names each entry and marks the songs the project no longer has", () => {
    expect(entryRows(draftOf(saved), songs)).toEqual([
      { id: 1, position: 1, name: "Opener", duration: 100, missing: false },
      { id: 4, position: 2, name: "gone", duration: null, missing: true },
      { id: 5, position: 3, name: "Closer", duration: 20, missing: false },
    ]);
  });

  it("adds up only the songs that exist", () => {
    expect(totalDuration(entryRows(draftOf(saved), songs))).toBe(120);
  });

  it("finds songs by part of the name", () => {
    expect(matchingSongs(songs, " ball ").map((found) => found.id)).toEqual(["b"]);
    expect(matchingSongs(songs, "")).toHaveLength(3);
  });
});
