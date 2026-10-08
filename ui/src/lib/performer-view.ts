import type { Catalog, LinkView, SongInfo } from "./generated/protocol";
import { performerPhase, type PerformerView, type SongView } from "./performer";

const setlistName = (catalog: Catalog): string | null =>
  catalog.setlists.find((setlist) => setlist.id === catalog.active_setlist)?.name ?? null;

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

export function performerView(link: LinkView, problem: string | null = null): PerformerView {
  const { catalog, live, status } = link;
  const index = live?.current_song ?? null;
  const song = index === null ? undefined : catalog.songs[index];
  const nextIndex = live?.next_song ?? null;
  const next = nextIndex === null ? undefined : catalog.songs[nextIndex];
  const songPosition = song && live ? Math.max(0, live.position - song.start) : 0;
  const before = index === null ? [] : catalog.songs.slice(0, index);
  const elapsedBefore = before.reduce((sum, earlier) => sum + songLength(earlier), 0);
  return {
    phase: performerPhase(live?.phase ?? "Idle"),
    setlistName: setlistName(catalog),
    song: song ? songView(song, catalog) : null,
    hasPrevious: index !== null && index > 0,
    nextSong: next ? { name: next.name, duration: songLength(next) } : null,
    songPosition,
    totalElapsed: elapsedBefore + songPosition,
    totalDuration: catalog.songs.reduce((sum, each) => sum + songLength(each), 0),
    autoResume: live?.autoplay ?? true,
    countInOnMarker: live?.count_in ?? false,
    recordArmed: live?.record_armed ?? false,
    stats: { connected: status !== "NotRunning" && !problem, midiActive: false, cpu: 0 },
  };
}
