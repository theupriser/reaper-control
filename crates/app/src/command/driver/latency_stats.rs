//! What was measured about the extension's answers.

use std::time::Duration;

/// How fast the extension answered commands so far.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LatencyStats {
    /// Commands answered.
    pub answered: u64,
    /// The latest answer's delay.
    pub last: Duration,
    /// The slowest answer's delay.
    pub slowest: Duration,
}
