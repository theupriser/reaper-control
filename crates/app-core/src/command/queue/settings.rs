//! How the command queue behaves.

use std::time::Duration;

/// The limits of the command queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueSettings {
    /// An identical command inside this window after the previous one is dropped as a repeat.
    pub repeat_window: Duration,
    /// A command the extension has not answered after this long is reported as timed out.
    pub timeout: Duration,
    /// At most this many commands may wait for an answer; more are refused.
    pub capacity: usize,
}

impl Default for QueueSettings {
    fn default() -> Self {
        Self {
            repeat_window: Duration::from_millis(250),
            timeout: Duration::from_secs(5),
            capacity: 32,
        }
    }
}
