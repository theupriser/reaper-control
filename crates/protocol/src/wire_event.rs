use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Setting;

/// Something that happened and must not be lost (SPEC §2.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
pub enum WireEvent {
    /// The first Play of a performance.
    PerformanceStarted,
    /// A hand-over began.
    HandOverStarted {
        /// Song that ended.
        from: String,
        /// Song that follows.
        to: String,
    },
    /// The next song is playing.
    HandOverCompleted {
        /// Song now playing.
        song_id: String,
    },
    /// A hard-stop song ended and playback stopped.
    HardStopReached {
        /// The hard-stop song.
        song_id: String,
    },
    /// The last song is done.
    PerformanceFinished,
    /// A setting changed.
    SettingChanged {
        /// Which one.
        setting: Setting,
        /// Its new value.
        enabled: bool,
    },
    /// The playhead jumped on purpose.
    SeekPerformed {
        /// Where it went, in seconds.
        to: f64,
    },
    /// The user switched to another project (tab) or opened one; the performance started over on
    /// its songs.
    ProjectChanged,
    /// A command was refused.
    CommandRejected {
        /// Why, for the log and the UI.
        reason: String,
    },
}
