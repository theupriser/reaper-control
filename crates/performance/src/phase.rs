/// Where the performance is. The only phase vocabulary (SPEC §14.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Nothing has started yet.
    Idle,
    /// A song is playing.
    Playing,
    /// Playback is paused.
    Paused,
    /// The click is running before a cue jump.
    CountingIn,
    /// A song with a hard stop reached its end and playback halted.
    HardStopped,
    /// Moving from one song to the next.
    HandingOver,
    /// The last song ended. Terminal until restart.
    Finished,
}
