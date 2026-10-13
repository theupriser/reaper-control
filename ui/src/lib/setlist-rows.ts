import type { SetlistInfo, SongInfo } from "./generated/protocol";
import { songLength } from "./performer-view";
import { draftOf, type SetlistDraft } from "./setlist-edit";

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

export interface CardSummary {
  count: number;
  duration: number;
  preview: string[];
  more: number;
  missing: number;
}

const PREVIEW_SONGS = 3;

/** What a setlist card shows: the count, the play time of the songs the project has, the first names and how many are gone. */
export function cardSummary(setlist: SetlistInfo, projectSongs: SongInfo[]): CardSummary {
  const rows = entryRows(draftOf(setlist), projectSongs);
  return {
    count: rows.length,
    duration: totalDuration(rows),
    preview: rows.slice(0, PREVIEW_SONGS).map((row) => row.name),
    more: Math.max(rows.length - PREVIEW_SONGS, 0),
    missing: rows.filter((row) => row.missing).length,
  };
}
