import { describe, expect, it } from "vitest";
import { addNotice, expireNotices } from "./notices";

const warning = { key: "command", level: "Warning", text: "Command not sent" } as const;

describe("notices", () => {
  it("replaces a notice with the same key", () => {
    const first = addNotice([], warning, 0);
    const second = addNotice(first, { ...warning, text: "Other" }, 5);
    expect(second.map((n) => n.text)).toEqual(["Other"]);
  });

  it("keeps notices with different keys side by side", () => {
    const shown = addNotice(addNotice([], warning, 0), { key: "link", level: "Error", text: "Lost" }, 1);
    expect(shown).toHaveLength(2);
  });

  it("lets an error stay longer than an info", () => {
    const shown = addNotice(addNotice([], { key: "a", level: "Info", text: "a" }, 0), { key: "b", level: "Error", text: "b" }, 0);
    expect(expireNotices(shown, 5000).map((n) => n.key)).toEqual(["b"]);
    expect(expireNotices(shown, 15000)).toEqual([]);
  });
});
