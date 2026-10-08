// Generated from crates/protocol by `UPDATE_TYPES=1 cargo test -p protocol`. Do not edit.

/**
 * The only phase vocabulary (SPEC §14.2).
 */
export type Phase = "Idle" | "Playing" | "Paused" | "CountingIn" | "HardStopped" | "HandingOver" | "Finished";

/**
 * Everything the UI can ask for.
 */
export type Command = "Play" | "Pause" | "Stop" | "Next" | "Previous" | "RestartSong" | { "Seek": { 
/**
 * Seconds from the start of the song.
 */
position: number, 
/**
 * Start with a count-in (a click on a Cue while "Count-in when pressing marker" is on).
 */
count_in: boolean, } } | "ToggleAutoResume" | "ToggleCountInOnMarker" | "ToggleRecordArm" | { "SaveSetlist": { 
/**
 * Identity of the setlist.
 */
id: string, 
/**
 * Display name.
 */
name: string, 
/**
 * The entries in playing order.
 */
entries: Array<EntryInfo>, 
/**
 * The revision of the setlist the edit was made on.
 */
expected_revision: number, } } | { "SetActiveSetlist": { 
/**
 * Identity of the setlist, if any.
 */
id: string | null, } };

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
revision: number, 
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
revision: number, 
/**
 * Raised whenever a setlist changes.
 */
setlist_revision: number, 
/**
 * Songs in playing order: the played setlist's, or timeline order without one. `Live::current_song` indexes this list.
 */
songs: Array<SongInfo>, 
/**
 * Every song of the project in timeline order, whatever the played setlist is.
 */
project_songs: Array<SongInfo>, 
/**
 * Cues in timeline order.
 */
cues: Array<CueInfo>, 
/**
 * The setlists of the project.
 */
setlists: Array<SetlistInfo>, 
/**
 * The id of the setlist being played, if any.
 */
active_setlist: string | null, };

/**
 * The state that changes all the time. Pushed on change (at most ~30 Hz while playing)
 * and at least once a second as the heartbeat; a gap of more than a second means stale.
 */
export type Live = { 
/**
 * Counts up per sender run; the receiver drops anything not newer than the last one.
 */
sequence: number, 
/**
 * Sender clock in seconds, for latency figures only.
 */
timestamp: number, 
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
 * Index into `Catalog.songs` of the song the performance is on.
 */
current_song: number | null, 
/**
 * Index into `Catalog.songs` of the song that follows.
 */
next_song: number | null, 
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
catalog_revision: number, 
/**
 * Revision of the setlists this state belongs to.
 */
setlist_revision: number, };

/**
 * Whether the app is talking to the extension.
 */
export type LinkStatus = "NotRunning" | { "Connected": { 
/**
 * Version of the extension build.
 */
extension_version: string, } };

/**
 * Everything the UI shows about the link: the connection and the last state the extension pushed.
 */
export type LinkView = { 
/**
 * Connection to the extension.
 */
status: LinkStatus, 
/**
 * Last pushed live state; none while nothing has been pushed.
 */
live: Live | null, 
/**
 * Last pushed catalog; empty while nothing has been pushed.
 */
catalog: Catalog, };
