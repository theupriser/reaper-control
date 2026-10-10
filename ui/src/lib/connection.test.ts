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
    const badge = connectionBadge("NotRunning", { message: "The extension turned itself off", extension_outdated: false });
    expect(badge.label).toBe("The extension turned itself off");
    expect(badge.tone).toBe("error");
  });

  it("asks for an update when the connected extension is not the bundled version", () => {
    const status = { Connected: { extension_version: "0.0.0" } };
    expect(connectionBadge(status, null, "1.0.0")).toEqual({
      label: "Extension out of date",
      detail: "extension 0.0.0, click to update it",
      tone: "error",
    });
    expect(connectionBadge(status, null, "0.0.0").tone).toBe("ok");
  });

  it("asks for an update when the installed extension is older", () => {
    const badge = connectionBadge("NotRunning", { message: "A new version of the extension is available", extension_outdated: true });
    expect(badge.label).toBe("A new version of the extension is available");
    expect(badge.detail).toBe("click to update it");
  });

  it("shows a problem even while the connection still answers", () => {
    const badge = connectionBadge({ Connected: { extension_version: "0.0.0" } }, { message: "The extension turned itself off", extension_outdated: false });
    expect(badge.label).toBe("The extension turned itself off");
  });
});
