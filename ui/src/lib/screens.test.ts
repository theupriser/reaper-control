import { describe, expect, it } from "vitest";
import { screenLabel, screens } from "./screens";

describe("screens", () => {
  it("lists the five canvas screens in sidebar order", () => {
    expect(screens.map((s) => s.id)).toEqual(["player", "setlists", "checklist", "settings", "help"]);
  });

  it("labels a screen by id", () => {
    expect(screenLabel("checklist")).toBe("Pre-show check");
  });
});
