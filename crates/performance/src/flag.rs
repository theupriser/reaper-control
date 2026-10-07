/// A playback setting the performer can switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// Resume playing after Next, Previous and Restart.
    Autoplay,
    /// Count in when jumping to a cue.
    CountIn,
}
