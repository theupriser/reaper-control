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

let remembered: { key: string; rows: PlayerRow[] } | null = null;

/**
 * The songs of the played setlist in order, each marked by where the performance is. A state push
 * that moves only the position gives the same rows back, so the list is not drawn again.
 */
export function playerRows(catalog: Catalog, live: Live | null): PlayerRow[] {
  const current = live?.current_song ?? null;
  const next = live?.next_song ?? null;
  const key = `${catalog.revision}/${catalog.setlist_revision}/${catalog.active_setlist}/${catalog.songs.length}/${current}/${next}`;
  if (remembered?.key === key) return remembered.rows;
  const rows = buildRows(catalog, current, next);
  remembered = { key, rows };
  return rows;
}

function buildRows(catalog: Catalog, current: number | null, next: number | null): PlayerRow[] {
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
