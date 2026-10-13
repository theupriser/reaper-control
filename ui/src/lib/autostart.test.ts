import { describe, expect, it } from "vitest";
import { autostartView } from "./autostart";
import { emptyLink } from "./app-state";

const connected = (autostart: boolean) => ({ ...emptyLink, status: { Connected: { extension_version: "1.0.7" } }, catalog: { ...emptyLink.catalog, autostart } });

describe("autostart view", () => {
  it("follows the project while REAPER is connected", () => {
    expect(autostartView(connected(true), true)).toEqual({ on: true, available: true, problem: null });
    expect(autostartView(connected(false), true)).toEqual({ on: false, available: true, problem: null });
  });

  it("says where the app is not known yet, only when it matters", () => {
    expect(autostartView(connected(true), false).problem).toContain("Open the installed app once");
    expect(autostartView(connected(false), false).problem).toBeNull();
  });

  it("cannot be used without REAPER", () => {
    const view = autostartView({ ...connected(true), status: "NotRunning" }, true);
    expect(view).toMatchObject({ on: false, available: false });
    expect(view.problem).toContain("Connect to REAPER");
  });
});
