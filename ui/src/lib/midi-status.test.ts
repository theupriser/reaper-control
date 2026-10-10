import { describe, expect, it } from "vitest";
import type { SettingsView } from "./generated/protocol";
import { midiBadge } from "./midi-status";

const view = (enabled: boolean, wanted: string | null, devices: string[]): SettingsView =>
  ({ settings: { midi_enabled: enabled, midi_device_name: wanted }, devices, actions: [] }) as unknown as SettingsView;

describe("midiBadge", () => {
  it("is hidden when MIDI is off", () => {
    expect(midiBadge(view(false, "Pad", ["Pad"]))).toBeNull();
  });

  it("is connected when the chosen device is present", () => {
    expect(midiBadge(view(true, "Pad", ["Pad"]))).toEqual({ label: "MIDI connected", detail: "Pad", tone: "ok" });
  });

  it("names the chosen device when it is not there", () => {
    expect(midiBadge(view(true, "Pad", []))).toEqual({ label: "MIDI not connected", detail: "Pad", tone: "error" });
  });

  it("looks at every device when none is chosen", () => {
    expect(midiBadge(view(true, null, []))?.label).toBe("No MIDI found");
    expect(midiBadge(view(true, null, ["A"]))?.detail).toBe("A");
    expect(midiBadge(view(true, null, ["A", "B", "C"]))?.detail).toBe("A and 2 more");
  });
});
