import type { SettingsView } from "./generated/protocol";
import type { ConnectionBadge } from "./connection";
import { strings } from "./strings";

/** The MIDI controller line of the sidebar; `null` when MIDI is switched off. */
export function midiBadge(view: SettingsView): ConnectionBadge | null {
  const { midi_enabled: enabled, midi_device_name: wanted } = view.settings;
  if (!enabled) return null;
  if (wanted !== null) {
    return view.devices.includes(wanted)
      ? { label: strings.midiStatus.connected, detail: wanted, tone: "ok" }
      : { label: strings.midiStatus.notConnected, detail: wanted, tone: "error" };
  }
  const [first, ...others] = view.devices;
  if (first === undefined) return { label: strings.midiStatus.noneFound, detail: strings.midiStatus.connectIt, tone: "error" };
  const detail = others.length === 0 ? first : strings.midiStatus.plusMore(first, others.length);
  return { label: strings.midiStatus.connected, detail, tone: "ok" };
}
