import { describe, expect, it } from "vitest";
import type { SettingsView } from "./generated/protocol";
import { midiLine } from "./midi-status";

const view = (enabled: boolean, wanted: string | null, devices: string[]): SettingsView =>
  ({ settings: { midi_enabled: enabled, midi_device_name: wanted }, devices, actions: [] }) as unknown as SettingsView;

describe("midiLine", () => {
  it("is hidden when MIDI is off", () => {
    expect(midiLine(view(false, "Pad", ["Pad"]))).toBeNull();
  });

  it("names the chosen device and says whether it is there", () => {
    expect(midiLine(view(true, "Pad", ["Pad"]))).toEqual({ text: "Pad connected", tone: "ok" });
    expect(midiLine(view(true, "Pad", []))).toEqual({ text: "Pad not connected", tone: "error" });
  });

  it("looks at every device when none is chosen", () => {
    expect(midiLine(view(true, null, []))).toEqual({ text: "No MIDI device", tone: "error" });
    expect(midiLine(view(true, null, ["A"]))).toEqual({ text: "A connected", tone: "ok" });
    expect(midiLine(view(true, null, ["A", "B", "C"]))?.text).toBe("A and 2 more connected");
  });
});
