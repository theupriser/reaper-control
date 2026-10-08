import type { Settings } from "./generated/protocol";

/** The form's fields as typed: numbers stay text until they are saved. */
export interface SettingsDraft {
  repeatWindow: string;
  timeout: string;
  capacity: string;
  midiEnabled: boolean;
  device: string;
  channel: string;
  debounce: string;
}

export const ALL_CHANNELS = "all";

export const toDraft = (settings: Settings): SettingsDraft => ({
  repeatWindow: String(settings.queue_repeat_window_milliseconds),
  timeout: String(settings.queue_timeout_milliseconds),
  capacity: String(settings.queue_capacity),
  midiEnabled: settings.midi_enabled,
  device: settings.midi_device_name ?? "",
  channel: settings.midi_channel === null ? ALL_CHANNELS : String(settings.midi_channel),
  debounce: String(settings.midi_debounce_milliseconds),
});

const whole = (text: string, label: string): number | string => {
  const trimmed = text.trim();
  return /^\d+$/.test(trimmed) ? Number(trimmed) : `${label} must be a whole number`;
};

/** The settings to save, or the first thing wrong with the typed values. Ranges are checked by the app. */
export function toSettings(draft: SettingsDraft): Settings | string {
  const repeat = whole(draft.repeatWindow, "Repeat window");
  const timeout = whole(draft.timeout, "Timeout");
  const capacity = whole(draft.capacity, "Queue size");
  const debounce = whole(draft.debounce, "Debounce");
  for (const value of [repeat, timeout, capacity, debounce]) {
    if (typeof value === "string") return value;
  }
  return {
    queue_repeat_window_milliseconds: repeat as number,
    queue_timeout_milliseconds: timeout as number,
    queue_capacity: capacity as number,
    midi_enabled: draft.midiEnabled,
    midi_device_name: draft.device === "" ? null : draft.device,
    midi_channel: draft.channel === ALL_CHANNELS ? null : Number(draft.channel),
    midi_debounce_milliseconds: debounce as number,
  };
}

export const isChanged = (draft: SettingsDraft, saved: Settings): boolean =>
  JSON.stringify(draft) !== JSON.stringify(toDraft(saved));
