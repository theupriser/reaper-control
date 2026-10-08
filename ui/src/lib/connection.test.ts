import { describe, expect, it } from "vitest";
import { connectionBadge } from "./connection";

describe("connectionBadge", () => {
  it("warns when REAPER is not running", () => {
    expect(connectionBadge("NotRunning")).toEqual({
      label: "REAPER not running",
      detail: "waiting for the extension",
      tone: "warn",
    });
  });

  it("shows the extension version when connected", () => {
    const badge = connectionBadge({ Connected: { extension_version: "0.0.0" } });
    expect(badge.tone).toBe("ok");
    expect(badge.detail).toBe("extension 0.0.0");
  });
});
