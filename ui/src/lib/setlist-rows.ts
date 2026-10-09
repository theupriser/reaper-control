import type { SongInfo } from "./generated/protocol";
import { songLength } from "./performer-view";
import type { SetlistDraft } from "./setlist-edit";

export interface EntryRow {
  id: number;
  position: number;
  name: string;
  duration: number | null;
  missing: boolean;
}

/** The entries of a draft with the name and length of their song; a song the project no longer has is `missing`. */
export function entryRows(draft: SetlistDraft, projectSongs: SongInfo[]): EntryRow[] {
  const known = new Map(projectSongs.map((song) => [song.id, song]));
  return draft.entries.map((entry, index) => {
    const song = known.get(entry.song_id);
    return {
      id: entry.id,
      position: index + 1,
      name: song?.name ?? entry.song_id,
      duration: song ? songLength(song) : null,
      missing: song === undefined,
    };
  });
}

/** The play time of the songs the project still has. */
export const totalDuration = (rows: EntryRow[]): number => rows.reduce((sum, row) => sum + (row.duration ?? 0), 0);

/** The songs whose name contains `query`, ignoring case and surrounding spaces. */
export function matchingSongs(songs: SongInfo[], query: string): SongInfo[] {
  const wanted = query.trim().toLowerCase();
  return wanted === "" ? songs : songs.filter((song) => song.name.toLowerCase().includes(wanted));
}
