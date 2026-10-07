// Generated from crates/protocol by `UPDATE_TYPES=1 cargo test -p protocol`. Do not edit.

/**
 * The only phase vocabulary (SPEC §14.2).
 */
export type Phase = "Idle" | "Playing" | "Paused" | "CountingIn" | "HardStopped" | "HandingOver" | "Finished";

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

/**
 * What REAPER's transport is doing.
 */
export type Transport = "Stopped" | "Playing" | "Paused" | "Recording";

/**
 * A playback setting that can change.
 */
export type Setting = "Autoplay" | "CountIn";

/**
 * Something that happened and must not be lost (SPEC §2.2).
 */
export type WireEvent = { "kind": "PerformanceStarted" } | { "kind": "HandOverStarted", 
/**
 * Song that ended.
 */
from: string, 
/**
 * Song that follows.
 */
to: string, } | { "kind": "HandOverCompleted", 
/**
 * Song now playing.
 */
song_id: string, } | { "kind": "HardStopReached", 
/**
 * The hard-stop song.
 */
song_id: string, } | { "kind": "PerformanceFinished" } | { "kind": "SettingChanged", 
/**
 * Which one.
 */
setting: Setting, 
/**
 * Its new value.
 */
enabled: boolean, } | { "kind": "SeekPerformed", 
/**
 * Where it went, in seconds.
 */
to: number, } | { "kind": "CommandRejected", 
/**
 * Why, for the log and the UI.
 */
reason: string, };

/**
 * An event with its place in the order of events.
 */
export type EventRecord = { 
/**
 * Starts at 1 and rises by one per event, so a receiver can tell what it missed.
 */
id: number, 
/**
 * What happened.
 */
event: WireEvent, };

/**
 * A song (a region) with the meaning of its markers already worked out.
 */
export type SongInfo = { 
/**
 * Identity that survives renames and moves (ADR-008).
 */
id: string, 
/**
 * Display name.
 */
name: string, 
/**
 * Start on the project timeline, in seconds.
 */
start: number, 
/**
 * End on the project timeline, in seconds.
 */
end: number, 
/**
 * Region colour as `#rrggbb`, if it has one.
 */
colour: string | null, 
/**
 * A `!1008` cue inside the song.
 */
hard_stop: boolean, 
/**
 * A `!length:N` cue inside the song, in seconds.
 */
length: number | null, 
/**
 * A `!bpm:N` cue inside the song.
 */
bpm: number | null, };

/**
 * A marker that is meant for navigation.
 */
export type CueInfo = { 
/**
 * Identity of the marker.
 */
id: string, 
/**
 * Display name.
 */
name: string, 
/**
 * Position on the project timeline, in seconds.
 */
position: number, };

/**
 * One place in a setlist.
 */
export type EntryInfo = { 
/**
 * Unique within the setlist and never reused.
 */
id: number, 
/**
 * The song played here.
 */
song_id: string, };

/**
 * A setlist as stored in the project.
 */
export type SetlistInfo = { 
/**
 * Identity of the setlist.
 */
id: string, 
/**
 * Display name.
 */
name: string, 
/**
 * Raised by one on every accepted edit; saving needs the revision you last saw.
 */
rev: number, 
/**
 * The entries in playing order.
 */
entries: Array<EntryInfo>, };

/**
 * What the project contains. Sent when a revision changes or when asked for.
 */
export type Catalog = { 
/**
 * Raised whenever songs or cues change.
 */
rev: number, 
/**
 * Raised whenever a setlist changes.
 */
setlist_rev: number, 
/**
 * Songs in timeline order.
 */
songs: Array<SongInfo>, 
/**
 * Cues in timeline order.
 */
cues: Array<CueInfo>, 
/**
 * The setlists of the project.
 */
setlists: Array<SetlistInfo>, };

/**
 * The state that changes all the time. Pushed on change (at most ~30 Hz while playing)
 * and at least once a second as the heartbeat; a gap of more than a second means stale.
 */
export type Live = { 
/**
 * Counts up per sender run; the receiver drops anything not newer than the last one.
 */
seq: number, 
/**
 * Sender clock in seconds, for latency figures only.
 */
ts: number, 
/**
 * What REAPER's transport does.
 */
transport: Transport, 
/**
 * Seconds on the project timeline.
 */
position: number, 
/**
 * Where the performance is.
 */
phase: Phase, 
/**
 * The active setlist, if one is selected.
 */
setlist_id: string | null, 
/**
 * Entry that is playing now.
 */
current_entry: number | null, 
/**
 * Entry that follows.
 */
next_entry: number | null, 
/**
 * "Auto-resume playback".
 */
autoplay: boolean, 
/**
 * "Count-in when pressing marker".
 */
count_in: boolean, 
/**
 * Recording is armed.
 */
record_armed: boolean, 
/**
 * Revision of the catalog this state belongs to.
 */
catalog_rev: number, 
/**
 * Revision of the setlists this state belongs to.
 */
setlist_rev: number, };
