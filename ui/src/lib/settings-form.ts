import type { Settings } from "./generated/protocol";
import { strings } from "./strings";

/** The form's fields as typed: numbers stay text until they are saved. */
export interface SettingsDraft {
  repeatWindow: string;
  timeout: string;
  capacity: string;
  midiEnabled: boolean;
  device: string;
  channel: string;
  debounce: string;
  logLevel: string;
}

export const ALL_CHANNELS = "all";

export const LOG_LEVELS = ["error", "warn", "info", "debug", "trace"];

export const toDraft = (settings: Settings): SettingsDraft => ({
  repeatWindow: String(settings.queue_repeat_window_milliseconds),
  timeout: String(settings.queue_timeout_milliseconds),
  capacity: String(settings.queue_capacity),
  midiEnabled: settings.midi_enabled,
  device: settings.midi_device_name ?? "",
  channel: settings.midi_channel === null ? ALL_CHANNELS : String(settings.midi_channel),
  debounce: String(settings.midi_debounce_milliseconds),
  logLevel: settings.log_level,
});

const whole = (text: string, label: string): number | string => {
  const trimmed = text.trim();
  return /^\d+$/.test(trimmed) ? Number(trimmed) : strings.settings.mustBeWhole(label);
};

/** The settings to save, or the first thing wrong with the typed values. Ranges are checked by the app. */
export function toSettings(draft: SettingsDraft): Settings | string {
  const repeat = whole(draft.repeatWindow, strings.settings.fields.repeatWindow);
  const timeout = whole(draft.timeout, strings.settings.fields.timeout);
  const capacity = whole(draft.capacity, strings.settings.fields.queueSize);
  const debounce = whole(draft.debounce, strings.settings.fields.debounce);
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
    log_level: draft.logLevel,
  };
}

export const isChanged = (draft: SettingsDraft, saved: Settings): boolean =>
  JSON.stringify(draft) !== JSON.stringify(toDraft(saved));
