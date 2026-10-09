import type { Command, EntryInfo, SetlistInfo } from "./generated/protocol";

/** A setlist being edited; `revision` is the one it was opened at (0 = not saved yet). */
export interface SetlistDraft {
  id: string;
  name: string;
  entries: EntryInfo[];
  revision: number;
  nextEntryId: number;
}

const nextIdAfter = (entries: EntryInfo[]): number => entries.reduce((highest, entry) => Math.max(highest, entry.id), 0) + 1;

export const draftOf = (setlist: SetlistInfo): SetlistDraft => ({
  id: setlist.id,
  name: setlist.name,
  entries: setlist.entries,
  revision: setlist.revision,
  nextEntryId: nextIdAfter(setlist.entries),
});

/** A short id from the name that none of `taken` uses. */
export function idFromName(name: string, taken: string[]): string {
  const base = name.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "setlist";
  let candidate = base;
  for (let n = 2; taken.includes(candidate); n += 1) candidate = `${base}-${n}`;
  return candidate;
}

export const emptyDraft = (name: string, taken: string[]): SetlistDraft => ({
  id: idFromName(name, taken),
  name,
  entries: [],
  revision: 0,
  nextEntryId: 1,
});

export const rename = (draft: SetlistDraft, name: string): SetlistDraft => ({ ...draft, name });

export const addSong = (draft: SetlistDraft, songId: string): SetlistDraft => ({
  ...draft,
  entries: [...draft.entries, { id: draft.nextEntryId, song_id: songId }],
  nextEntryId: draft.nextEntryId + 1,
});

export const removeEntry = (draft: SetlistDraft, entryId: number): SetlistDraft => ({
  ...draft,
  entries: draft.entries.filter((entry) => entry.id !== entryId),
});

/** Moves an entry up (-1) or down (1); at either end it stays where it is. */
export function moveEntry(draft: SetlistDraft, entryId: number, direction: -1 | 1): SetlistDraft {
  const from = draft.entries.findIndex((entry) => entry.id === entryId);
  const to = from + direction;
  if (from < 0 || to < 0 || to >= draft.entries.length) return draft;
  const entries = [...draft.entries];
  [entries[from], entries[to]] = [entries[to], entries[from]];
  return { ...draft, entries };
}

export function isChanged(draft: SetlistDraft, saved: SetlistInfo | undefined): boolean {
  if (!saved) return true;
  return (
    draft.name.trim() !== saved.name ||
    draft.entries.length !== saved.entries.length ||
    draft.entries.some((entry, at) => entry.id !== saved.entries[at]?.id || entry.song_id !== saved.entries[at]?.song_id)
  );
}

/** Why the draft cannot be saved, or null. */
export function saveProblem(draft: SetlistDraft): "name" | null {
  return draft.name.trim() === "" ? "name" : null;
}

export const saveCommand = (draft: SetlistDraft): Command => ({
  SaveSetlist: { id: draft.id, name: draft.name.trim(), entries: draft.entries, expected_revision: draft.revision },
});

export const deleteCommand = (draft: SetlistDraft): Command => ({
  DeleteSetlist: { id: draft.id, expected_revision: draft.revision },
});
