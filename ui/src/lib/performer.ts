import type { Command, Phase } from "./generated/protocol";

// Temporary hand-written view model for the Performer screen.
// Replaced by generated types in WP 0.4; the performance core will fill it.
export type PerformerPhase = "Idle" | "Playing" | "Paused" | "CountingIn" | "HardStopped";

// The Performer screen (v1 look) has no state of its own for a hand-over or a finished show:
// a hand-over looks like playing, a finished show like idle.
export function performerPhase(phase: Phase): PerformerPhase {
  switch (phase) {
    case "HandingOver":
      return "Playing";
    case "Finished":
      return "Idle";
    default:
      return phase;
  }
}

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

export interface SystemStatsView {
  connected: boolean;
  midiActive: boolean;
  cpu: number;
}

export interface PerformerView {
  phase: PerformerPhase;
  setlistName: string | null;
  song: SongView | null;
  nextSong: { name: string; duration: number } | null;
  hasPrevious: boolean;
  songPosition: number;
  totalElapsed: number;
  totalDuration: number;
  autoResume: boolean;
  countInOnMarker: boolean;
  recordArmed: boolean;
  stats: SystemStatsView;
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

export type UsageLevel = "low" | "medium" | "high";

export function usageLevel(percent: number): UsageLevel {
  if (percent < 50) return "low";
  if (percent < 80) return "medium";
  return "high";
}

// v1: a click within 10 px of a cue seeks to the cue itself.
export const CUE_SNAP_PX = 10;
const POPOVER_HALF_WIDTH = 30;

export interface SeekTarget {
  position: number;
  onCue: boolean;
}

export function seekTarget(clickX: number, width: number, song: SongView): SeekTarget {
  if (width <= 0 || song.duration <= 0) return { position: 0, onCue: false };
  const fraction = Math.min(1, Math.max(0, clickX / width));
  const cue = song.cues.find((c) => Math.abs(clickX - (c.position / song.duration) * width) <= CUE_SNAP_PX);
  return cue ? { position: cue.position, onCue: true } : { position: fraction * song.duration, onCue: false };
}

export function popoverX(clickX: number, width: number): number {
  return Math.min(Math.max(clickX, POPOVER_HALF_WIDTH), Math.max(POPOVER_HALF_WIDTH, width - POPOVER_HALF_WIDTH));
}

export function seekCommand(target: SeekTarget, countInOnMarker: boolean): Command {
  return { Seek: { position: target.position, count_in: target.onCue && countInOnMarker } };
}

export type KeyIntent = "PlayPause" | "ToggleAutoResume" | "Previous" | "Next";

export function keyIntent(key: string): KeyIntent | null {
  if (key === " ") return "PlayPause";
  if (key === "a") return "ToggleAutoResume";
  if (key === "ArrowLeft") return "Previous";
  if (key === "ArrowRight") return "Next";
  return null;
}
