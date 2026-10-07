// Generated from crates/protocol by `UPDATE_TYPES=1 cargo test -p protocol`. Do not edit.

/**
 * Phases the stub knows about; a subset of SPEC §14.2.
 */
export type Phase = "Idle" | "Playing" | "Paused";

/**
 * Everything the UI can ask for.
 */
export type Command = "Play" | "Pause" | "Stop" | { "Seek": { 
/**
 * Seconds from the start of the song.
 */
position: number, 
/**
 * Start with a count-in (a click on a Cue while "Count-in when pressing marker" is on).
 */
count_in: boolean, } } | "ToggleAutoResume" | "ToggleCountInOnMarker" | "ToggleRecordArm";

/**
 * What the UI renders. The UI never infers it.
 */
export type AppState = { 
/**
 * Current phase.
 */
phase: Phase, 
/**
 * Seconds from the start of the current song.
 */
position: number, 
/**
 * "Auto-resume playback".
 */
auto_resume: boolean, 
/**
 * "Count-in when pressing marker".
 */
count_in_on_marker: boolean, 
/**
 * Recording is armed.
 */
record_armed: boolean, };
