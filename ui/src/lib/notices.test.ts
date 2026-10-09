import { describe, expect, it } from "vitest";
import { addNotice, dismissNotice, expireNotices, noticeTone } from "./notices";

const warning = { key: "command", level: "Warning", title: "Command not sent", text: "Nope" } as const;

describe("notices", () => {
  it("replaces a notice with the same key", () => {
    const first = addNotice([], warning, 0);
    const second = addNotice(first, { ...warning, title: "Other", text: "Other" }, 5);
    expect(second.map((n) => n.text)).toEqual(["Other"]);
  });

  it("keeps notices with different keys side by side", () => {
    const shown = addNotice(addNotice([], warning, 0), { key: "link", level: "Error", title: "Lost", text: "Lost" }, 1);
    expect(shown).toHaveLength(2);
  });

  it("keeps an error until it is dismissed", () => {
    const shown = addNotice(addNotice([], { key: "a", level: "Info", title: "a", text: "a" }, 0), { key: "b", level: "Error", title: "b", text: "b" }, 0);
    expect(expireNotices(shown, 2000).map((n) => n.key)).toEqual(["b"]);
    expect(expireNotices(shown, 3_600_000).map((n) => n.key)).toEqual(["b"]);
    expect(dismissNotice(shown, "b").map((n) => n.key)).toEqual(["a"]);
  });

  it("clears a warning after ten seconds", () => {
    const shown = addNotice([], warning, 0);
    expect(expireNotices(shown, 9999)).toHaveLength(1);
    expect(expireNotices(shown, 10000)).toEqual([]);
  });
});

describe("noticeTone", () => {
  it("maps each level to a toast tone", () => {
    expect(["Info", "Warning", "Error"].map((level) => noticeTone(level as "Info"))).toEqual(["info", "warn", "error"]);
  });
});
