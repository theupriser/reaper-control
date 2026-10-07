use shared_kernel::Seconds;

/// What the adapter must do in REAPER. Effects are idempotent requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    /// Start or continue playback.
    Play,
    /// Halt playback.
    Pause,
    /// Move the play position, also while playing.
    SeekTo(Seconds),
    /// Turn REAPER's count-in on or off.
    SetCountIn(bool),
}
