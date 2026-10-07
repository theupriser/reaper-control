use shared_kernel::Seconds;

use crate::Flag;

/// Everything that can happen to a performance: commands and the clock tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Input {
    /// Time passed. `now` is the injected clock, `position` where REAPER plays.
    Tick {
        /// Monotonic time.
        now: Seconds,
        /// Play position on the timeline.
        position: Seconds,
    },
    /// Start or resume playing.
    Play,
    /// Pause.
    Pause,
    /// Go to the next song.
    Next,
    /// Go to the previous song.
    Previous,
    /// Go to the start of the current song.
    RestartSong,
    /// Jump inside the current song, never with a count-in.
    Seek {
        /// Target position.
        position: Seconds,
    },
    /// Jump to a cue inside the current song, with a count-in when enabled.
    SeekCue {
        /// Cue position.
        position: Seconds,
        /// Length of the count-in (two bars at the tempo there).
        lead_in: Seconds,
    },
    /// Switch a playback setting.
    SetFlag {
        /// Which setting.
        flag: Flag,
        /// New value.
        enabled: bool,
    },
}
