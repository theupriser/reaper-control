// Temporary hand-written view model for the Performer screen.
// Replaced by generated types in WP 0.4; the performance core will fill it.
export type PerformerPhase = "Idle" | "Playing" | "Paused" | "CountingIn" | "HardStopped";

export interface CueMark {
  name: string;
  position: number;
}

export interface SongView {
  name: string;
  duration: number;
  hardStop: boolean;
  cues: CueMark[];
}

export interface PerformerView {
  phase: PerformerPhase;
  setlistName: string | null;
  song: SongView | null;
  nextSong: { name: string; duration: number } | null;
  songPosition: number;
  totalElapsed: number;
  totalDuration: number;
  autoResume: boolean;
  countInOnMarker: boolean;
}

const pad = (n: number) => String(n).padStart(2, "0");

export function formatTime(seconds: number): string {
  const whole = Math.max(0, Math.floor(seconds));
  return `${Math.floor(whole / 60)}:${pad(whole % 60)}`;
}

export function formatLongTime(seconds: number): string {
  const whole = Math.max(0, Math.floor(seconds));
  const h = Math.floor(whole / 3600);
  return `${h}:${pad(Math.floor((whole % 3600) / 60))}:${pad(whole % 60)}`;
}

export function progressPercent(position: number, duration: number): number {
  if (duration <= 0) return 0;
  return Math.min(100, Math.max(0, (position / duration) * 100));
}

export function isWaitingAtHardStop(view: PerformerView): boolean {
  return view.phase === "HardStopped" && view.nextSong !== null;
}
