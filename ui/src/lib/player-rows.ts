import type { Catalog, Live } from "./generated/protocol";
import { songLength } from "./performer-view";

export type RowState = "played" | "current" | "next" | "upcoming";

export interface PlayerRow {
  id: string;
  number: number;
  name: string;
  duration: number;
  bpm: number | null;
  hardStop: boolean;
  state: RowState;
}

/** The songs of the played setlist in order, each marked by where the performance is. */
export function playerRows(catalog: Catalog, live: Live | null): PlayerRow[] {
  const current = live?.current_song ?? null;
  const next = live?.next_song ?? null;
  return catalog.songs.map((song, index) => ({
    id: song.id,
    number: index + 1,
    name: song.name,
    duration: songLength(song),
    bpm: song.bpm,
    hardStop: song.hard_stop,
    state: index === current ? "current" : index === next ? "next" : current !== null && index < current ? "played" : "upcoming",
  }));
}
