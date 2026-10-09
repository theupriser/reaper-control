import { describe, expect, it } from "vitest";
import { contrastRatio, themeColours } from "./contrast";
import { themes } from "./appearance";
import rawCss from "../tokens.css?raw";

const css = rawCss.replace(/\r\n/g, "\n");

/** Text needs 4.5:1 (WCAG AA); status colours and borders that carry meaning need 3:1. */
const text: [string, string][] = [
  ["--text", "--bg"], ["--text", "--panel"], ["--text", "--raised"], ["--text", "--sidebar"],
  ["--text-soft", "--panel"], ["--muted", "--bg"], ["--muted", "--panel"], ["--muted", "--raised"],
  ["--green", "--panel"], ["--amber", "--panel"], ["--red", "--panel"], ["--info", "--panel"],
  ["--on-accent", "--green"],
];
const graphics: [string, string][] = [["--info", "--bg"], ["--green", "--bg"], ["--red", "--bg"]];

describe("contrast", () => {
  it("computes known ratios", () => {
    expect(contrastRatio("#000", "#fff")).toBeCloseTo(21, 0);
    expect(contrastRatio("#777", "#777")).toBe(1);
  });

  for (const theme of themes) {
    const colours = themeColours(css, theme);
    const ratio = ([front, back]: [string, string]) => contrastRatio(colours[front], colours[back]);

    it(`${theme}: text pairs reach 4.5:1`, () => {
      for (const pair of text) expect(ratio(pair), `${theme} ${pair.join(" on ")}`).toBeGreaterThanOrEqual(4.5);
    });

    it(`${theme}: focus ring and status colours reach 3:1`, () => {
      for (const pair of graphics) expect(ratio(pair), `${theme} ${pair.join(" on ")}`).toBeGreaterThanOrEqual(3);
    });
  }
});
