//! What the chaos proxy does to the traffic from the extension to the app.

use std::time::Duration;

/// The faults to apply. All off by default; counts are per connection, so the same settings
/// give the same behaviour on every run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChaosSettings {
    /// Wait this long before passing on each frame.
    pub delay: Duration,
    /// Pass frames on in swapped pairs: the second of two neighbours arrives first.
    pub swap_neighbours: bool,
    /// Once this many frames have passed, send bytes that are not a frame (once per connection).
    pub garbage_after_frames: Option<u32>,
    /// After this many frames, close the connection.
    pub cut_after_frames: Option<u32>,
}
