// Stand-in data until the performance core feeds the screen (Phase 2).
import type { PerformerPhase, PerformerView } from "./performer";

const song = {
  name: "Example Song",
  duration: 215,
  hardStop: true,
  cues: [
    { name: "Verse", position: 20 },
    { name: "Chorus", position: 64 },
    { name: "Bridge", position: 130 },
  ],
};

const base: PerformerView = {
  phase: "Idle",
  setlistName: "Example Setlist",
  song,
  nextSong: { name: "Another Song", duration: 187 },
  songPosition: 0,
  totalElapsed: 0,
  totalDuration: 1325,
  autoResume: true,
  countInOnMarker: false,
  recordArmed: false,
  stats: { connected: true, midiActive: false, cpu: 23 },
};

const positions: Record<PerformerPhase, number> = {
  Idle: 0,
  Playing: 72,
  Paused: 72,
  CountingIn: 64,
  HardStopped: 215,
};

export function fixtureFor(phase: PerformerPhase): PerformerView {
  const songPosition = positions[phase];
  return { ...base, phase, songPosition, totalElapsed: songPosition };
}
