/// What REAPER's transport is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// Not started yet, or stopped.
    Stopped,
    /// Playing.
    Playing,
    /// Paused.
    Paused,
}
