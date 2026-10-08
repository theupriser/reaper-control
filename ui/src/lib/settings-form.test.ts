import { describe, expect, it } from "vitest";
import type { Settings } from "./generated/protocol";
import { isChanged, toDraft, toSettings } from "./settings-form";

const saved: Settings = {
  queue_repeat_window_milliseconds: 300,
  queue_timeout_milliseconds: 5000,
  queue_capacity: 16,
  midi_enabled: true,
  midi_device_name: null,
  midi_channel: null,
  midi_debounce_milliseconds: 200,
};

describe("settings form", () => {
  it("round-trips the saved values", () => {
    expect(toSettings(toDraft(saved))).toEqual(saved);
  });

  it("maps 'all' and an empty device to nothing, and a channel to its number", () => {
    const draft = { ...toDraft(saved), device: "FootCtrl Mini", channel: "3" };
    expect(toSettings(draft)).toMatchObject({ midi_device_name: "FootCtrl Mini", midi_channel: 3 });
  });

  it("names the field that is not a whole number", () => {
    expect(toSettings({ ...toDraft(saved), timeout: "fast" })).toBe("Timeout must be a whole number");
    expect(toSettings({ ...toDraft(saved), capacity: "-1" })).toBe("Queue size must be a whole number");
    expect(toSettings({ ...toDraft(saved), debounce: "" })).toBe("Debounce must be a whole number");
  });

  it("sees whether the form differs from what is saved", () => {
    expect(isChanged(toDraft(saved), saved)).toBe(false);
    expect(isChanged({ ...toDraft(saved), midiEnabled: false }, saved)).toBe(true);
  });
});
