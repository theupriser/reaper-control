//! The port through which the app reads time.

use std::time::Duration;

/// A monotonic clock; the app never reads the real time directly so tests can move it.
pub trait Clock: Send + Sync {
    /// Time since an arbitrary fixed start; it never goes back.
    fn now(&self) -> Duration;
}
