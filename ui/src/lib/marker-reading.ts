import type { MarkerReading } from "./generated/protocol";
import { strings } from "./strings";

/** What the app made of a marker name, as lines of plain text for the Help screen. */
export const readingLines = (reading: MarkerReading): string[] => {
  const text = strings.help.helper;
  const lines: string[] = [];
  if (reading.hard_stop) lines.push(text.hardStop);
  if (reading.length_seconds !== null) lines.push(text.length(reading.length_seconds));
  if (reading.tempo_bpm !== null) lines.push(text.tempo(reading.tempo_bpm));
  if (reading.hidden) lines.push(text.hidden);
  return lines;
};
