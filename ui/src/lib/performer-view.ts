import type { Catalog, LinkView, SongInfo } from "./generated/protocol";
import { performerPhase, type PerformerView, type SongView } from "./performer";

// Without a setlist the songs play in timeline order (the setlist arrives with its own work package).
const SETLIST_NAME = null;

export const songLength = (song: SongInfo): number => song.length ?? song.end - song.start;

function songView(song: SongInfo, catalog: Catalog): SongView {
  return {
    name: song.name,
    duration: songLength(song),
    hardStop: song.hard_stop,
    cues: catalog.cues
      .filter((cue) => cue.position >= song.start && cue.position <= song.end)
      .map((cue) => ({ name: cue.name, position: cue.position - song.start })),
  };
}

export function performerView(link: LinkView): PerformerView {
  const { catalog, state, status } = link;
  const index = state.current_song;
  const song = index === null ? undefined : catalog.songs[index];
  const next = index === null ? undefined : catalog.songs[index + 1];
  const before = index === null ? [] : catalog.songs.slice(0, index);
  const elapsedBefore = before.reduce((sum, earlier) => sum + songLength(earlier), 0);
  return {
    phase: performerPhase(state.phase),
    setlistName: SETLIST_NAME,
    song: song ? songView(song, catalog) : null,
    hasPrevious: index !== null && index > 0,
    nextSong: next ? { name: next.name, duration: songLength(next) } : null,
    songPosition: state.position,
    totalElapsed: elapsedBefore + state.position,
    totalDuration: catalog.songs.reduce((sum, each) => sum + songLength(each), 0),
    autoResume: state.auto_resume,
    countInOnMarker: state.count_in_on_marker,
    recordArmed: state.record_armed,
    stats: { connected: status !== "NotRunning", midiActive: false, cpu: 0 },
  };
}
