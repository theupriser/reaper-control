//! The port through which the app controls REAPER.

mod error;
mod latency_stats;
mod metered;

pub use error::DriverError;
pub use latency_stats::LatencyStats;
pub use metered::MeteredDriver;

use protocol::Command;

/// What the app uses to make REAPER do something: the extension in production, `FakeDriver` in tests.
pub trait Driver: Send + Sync {
    /// Hands `command` over and returns the id its answer will carry; an error means it was not sent.
    fn send(&self, command: Command) -> Result<u64, DriverError>;
}
