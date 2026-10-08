import { describe, expect, it } from "vitest";
import { connectionBadge } from "./connection";

describe("connectionBadge", () => {
  it("is an error when REAPER is not running", () => {
    expect(connectionBadge("NotRunning")).toEqual({
      label: "REAPER not running",
      detail: "waiting for the extension",
      tone: "error",
    });
  });

  it("shows the extension version when connected", () => {
    const badge = connectionBadge({ Connected: { extension_version: "0.0.0" } });
    expect(badge.tone).toBe("ok");
    expect(badge.detail).toBe("extension 0.0.0");
  });

  it("names the problem the app found instead of guessing", () => {
    const badge = connectionBadge("NotRunning", "The extension turned itself off");
    expect(badge.label).toBe("The extension turned itself off");
    expect(badge.tone).toBe("error");
  });

  it("shows a problem even while the connection still answers", () => {
    const badge = connectionBadge({ Connected: { extension_version: "0.0.0" } }, "The extension turned itself off");
    expect(badge.label).toBe("The extension turned itself off");
  });
});
