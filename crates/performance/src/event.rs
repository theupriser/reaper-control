use shared_kernel::Seconds;

use crate::{Flag, Rejection};

/// A fact for the journal and the event stream.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Playing began from the idle or finished state.
    PerformanceStarted,
    /// The end of a song triggered the move to the next one.
    HandOverStarted {
        /// Song that ends.
        from: String,
        /// Song that follows.
        to: String,
    },
    /// The position is inside the next song.
    HandOverCompleted {
        /// Song now playing.
        song_id: String,
    },
    /// A hard stop halted playback.
    HardStopReached {
        /// The song that stopped.
        song_id: String,
    },
    /// The last song ended.
    PerformanceFinished,
    /// A setting changed.
    FlagChanged {
        /// Which setting.
        flag: Flag,
        /// New value.
        enabled: bool,
    },
    /// The position was moved on purpose.
    SeekPerformed {
        /// Where to.
        to: Seconds,
    },
    /// A command was refused.
    CommandRejected(Rejection),
}
