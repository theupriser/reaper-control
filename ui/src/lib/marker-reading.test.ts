import { describe, expect, it } from "vitest";
import { readingLines } from "./marker-reading";
import { strings } from "./strings";

const nothing = { hard_stop: false, length_seconds: null, tempo_bpm: null, hidden: false };

describe("readingLines", () => {
  it("says nothing for a plain name", () => {
    expect(readingLines(nothing)).toEqual([]);
  });

  it("names every directive it found", () => {
    const lines = readingLines({ hard_stop: true, length_seconds: 45, tempo_bpm: 140, hidden: true });
    expect(lines).toEqual([
      strings.help.helper.hardStop,
      strings.help.helper.length(45),
      strings.help.helper.tempo(140),
      strings.help.helper.hidden,
    ]);
  });
});
