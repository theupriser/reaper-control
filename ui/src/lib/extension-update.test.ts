import { describe, expect, it } from "vitest";
import { updateOffered } from "./extension-update";
import type { InstallationView } from "./generated/protocol";

const view = (update_available: boolean): InstallationView => ({
  steps: [],
  folder: "",
  can_install: update_available,
  complete: true,
  update_available,
  reaper_running: true,
});

describe("updateOffered", () => {
  it("offers a ready update when nothing plays", () => {
    expect(updateOffered(view(true), "Idle")).toBe(true);
    expect(updateOffered(view(true), "Finished")).toBe(true);
    expect(updateOffered(view(true), undefined)).toBe(true);
  });

  it("offers nothing without an update or before the installation is known", () => {
    expect(updateOffered(view(false), "Idle")).toBe(false);
    expect(updateOffered(null, "Idle")).toBe(false);
  });

  it("never offers during a show", () => {
    for (const phase of ["Playing", "Paused", "CountingIn", "HardStopped", "HandingOver"] as const) {
      expect(updateOffered(view(true), phase)).toBe(false);
    }
  });
});
