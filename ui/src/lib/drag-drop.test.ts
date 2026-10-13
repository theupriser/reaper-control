import { describe, expect, it } from "vitest";
import { insertionIndex } from "./drag-drop";

describe("insertion index", () => {
  const middles = [25, 75, 125];

  it("counts the rows above the pointer", () => {
    expect(insertionIndex(middles, 0)).toBe(0);
    expect(insertionIndex(middles, 50)).toBe(1);
    expect(insertionIndex(middles, 100)).toBe(2);
    expect(insertionIndex(middles, 500)).toBe(3);
  });

  it("is 0 for an empty list", () => {
    expect(insertionIndex([], 40)).toBe(0);
  });
});
