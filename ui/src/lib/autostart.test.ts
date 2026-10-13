import { describe, expect, it } from "vitest";
import { autostartView } from "./autostart";
import { emptyLink } from "./app-state";

const connected = (autostart: boolean) => ({ ...emptyLink, status: { Connected: { extension_version: "1.0.7" } }, catalog: { ...emptyLink.catalog, autostart } });

describe("autostart view", () => {
  it("follows the project while REAPER is connected", () => {
    expect(autostartView(connected(true))).toEqual({ on: true, available: true, problem: null });
    expect(autostartView(connected(false))).toEqual({ on: false, available: true, problem: null });
  });

  it("cannot be used without REAPER", () => {
    const view = autostartView({ ...connected(true), status: "NotRunning" });
    expect(view).toMatchObject({ on: false, available: false });
    expect(view.problem).toContain("Connect to REAPER");
  });
});
