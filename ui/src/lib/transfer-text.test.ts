import { describe, expect, it } from "vitest";
import { doneText, offerText } from "./transfer-text";

describe("transfer text", () => {
  it("names the songs a v1 setlist has no match for", () => {
    expect(offerText({ id: "a", name: "Friday", found: 2, missing: ["Gone", "Lost"] })).toBe(
      "2 songs; not in this project: Gone, Lost",
    );
  });

  it("keeps it short when everything is found", () => {
    expect(offerText({ id: "a", name: "Friday", found: 1, missing: [] })).toBe("1 song");
  });

  it("says what was done", () => {
    expect(doneText(2, "import")).toBe("2 setlists imported.");
    expect(doneText(1, "restore")).toBe("1 setlist restored.");
    expect(doneText(0, "import")).toBe("Nothing to import.");
  });
});
