import { describe, expect, it } from "vitest";
import type { SetlistInfo, SongInfo } from "./generated/protocol";
import { addSong, deleteCommand, draftOf, emptyDraft, idFromName, isChanged, moveEntry, removeEntry, rename, saveCommand, saveProblem } from "./setlist-edit";

const saved: SetlistInfo = {
  id: "sat",
  name: "Saturday",
  revision: 3,
  entries: [
    { id: 1, song_id: "a" },
    { id: 4, song_id: "b" },
    { id: 5, song_id: "c" },
  ],
};

const song = (id: string): SongInfo => ({ id, number: 1, name: id, start: 0, end: 10, colour: null, hard_stop: false, length: null, bpm: null });

describe("setlist edit", () => {
  it("opens a setlist with entry ids that are never reused", () => {
    const draft = addSong(removeEntry(draftOf(saved), 5), "d");
    expect(draft.entries.map((entry) => entry.id)).toEqual([1, 4, 6]);
  });

  it("moves an entry and stays put at the ends", () => {
    const draft = draftOf(saved);
    expect(moveEntry(draft, 4, -1).entries.map((entry) => entry.song_id)).toEqual(["b", "a", "c"]);
    expect(moveEntry(draft, 4, 1).entries.map((entry) => entry.song_id)).toEqual(["a", "c", "b"]);
    expect(moveEntry(draft, 1, -1)).toBe(draft);
    expect(moveEntry(draft, 5, 1)).toBe(draft);
    expect(moveEntry(draft, 99, 1)).toBe(draft);
  });

  it("knows what changed", () => {
    const draft = draftOf(saved);
    expect(isChanged(draft, saved)).toBe(false);
    expect(isChanged(rename(draft, "Sunday"), saved)).toBe(true);
    expect(isChanged(moveEntry(draft, 4, -1), saved)).toBe(true);
    expect(isChanged(emptyDraft("New", []), undefined)).toBe(true);
  });

  it("makes ids from names that no other setlist has", () => {
    expect(idFromName("Saturday Night!", [])).toBe("saturday-night");
    expect(idFromName("Saturday", ["saturday", "saturday-2"])).toBe("saturday-3");
    expect(idFromName("???", [])).toBe("setlist");
  });

  it("refuses an empty name", () => {
    expect(saveProblem(rename(draftOf(saved), "  "))).toBe("name");
    expect(saveProblem(draftOf(saved))).toBeNull();
  });

  it("saves with the revision it was opened at", () => {
    expect(saveCommand(rename(draftOf(saved), " Sunday "))).toEqual({
      SaveSetlist: { id: "sat", name: "Sunday", entries: saved.entries, expected_revision: 3 },
    });
    expect(saveCommand(emptyDraft("New one", ["sat"]))).toEqual({
      SaveSetlist: { id: "new-one", name: "New one", entries: [], expected_revision: 0 },
    });
  });
});

it("deleting names the revision the setlist was opened at", () => {
  const draft = draftOf({ id: "sat", name: "Saturday", revision: 3, entries: [] });
  expect(deleteCommand(draft)).toEqual({ DeleteSetlist: { id: "sat", expected_revision: 3 } });
});
