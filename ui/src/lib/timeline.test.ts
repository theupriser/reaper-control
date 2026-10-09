import { describe, expect, it } from "vitest";
import { markOf, percentOf } from "./timeline";

const span = { start: 10, end: 30 };

describe("timeline positions", () => {
  it("maps time to a percentage of the span", () => {
    expect(percentOf(10, span)).toBe(0);
    expect(percentOf(20, span)).toBe(50);
    expect(percentOf(30, span)).toBe(100);
  });

  it("clamps times outside the span", () => {
    expect(percentOf(0, span)).toBe(0);
    expect(percentOf(99, span)).toBe(100);
  });

  it("gives 0 for an empty span or a non-finite time", () => {
    expect(percentOf(5, { start: 5, end: 5 })).toBe(0);
    expect(percentOf(5, { start: 9, end: 5 })).toBe(0);
    expect(percentOf(Number.NaN, span)).toBe(0);
  });

  it("builds a labelled mark", () => {
    expect(markOf(25, "Chorus", span)).toEqual({ percent: 75, label: "Chorus" });
  });
});
