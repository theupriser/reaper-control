// Temporary hand-written mirror of the Rust `Command` and `AppState` in crates/app.
// Replaced by generated types in WP 0.4; do not extend.
export type Phase = "Idle" | "Playing" | "Paused";
export type Command = "Play" | "Pause" | "Stop";
export interface AppState {
  phase: Phase;
}
