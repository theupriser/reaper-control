// Generated from crates/protocol by `UPDATE_TYPES=1 cargo test -p protocol`. Do not edit.

/**
 * Phases the stub knows about; a subset of SPEC §14.2.
 */
export type Phase = "Idle" | "Playing" | "Paused";

/**
 * Everything the UI can ask for.
 */
export type Command = "Play" | "Pause" | "Stop";

/**
 * What the UI renders. The UI never infers it.
 */
export type AppState = { 
/**
 * Current phase.
 */
phase: Phase, };
