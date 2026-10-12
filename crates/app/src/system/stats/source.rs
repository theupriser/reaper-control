//! The port through which the app reads how busy the machine is.

use protocol::SystemStats;

/// Reads the operating system's counters.
pub trait StatsSource: Send {
    /// A fresh reading. CPU use needs two readings apart, so the first one reports 0.
    fn read(&mut self) -> SystemStats;
}
