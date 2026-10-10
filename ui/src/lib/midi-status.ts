import type { SettingsView } from "./generated/protocol";
import { strings } from "./strings";

export type MidiLine = { text: string; tone: "ok" | "error" };

/** The MIDI controller line of the sidebar; `null` when MIDI is switched off. */
export function midiLine(view: SettingsView): MidiLine | null {
  const { midi_enabled: enabled, midi_device_name: wanted } = view.settings;
  if (!enabled) return null;
  if (wanted !== null) {
    const present = view.devices.includes(wanted);
    return { text: `${wanted} ${present ? strings.midiStatus.connected : strings.midiStatus.notConnected}`, tone: present ? "ok" : "error" };
  }
  const [first, ...others] = view.devices;
  if (first === undefined) return { text: strings.midiStatus.noDevice, tone: "error" };
  const name = others.length === 0 ? first : strings.midiStatus.plusMore(first, others.length);
  return { text: `${name} ${strings.midiStatus.connected}`, tone: "ok" };
}
