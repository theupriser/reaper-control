//! The port through which the app controls REAPER.

use protocol::Command;

use crate::driver_error::DriverError;

/// What the app uses to make REAPER do something: the extension in production, `FakeDriver` in tests.
pub trait Driver: Send + Sync {
    /// Hands `command` over; an error means it was not sent.
    fn send(&self, command: Command) -> Result<(), DriverError>;
}
