//! What the app announces about itself.

use protocol::Command;

use crate::driver_error::DriverError;

/// Something that happened inside the app, for anyone who listens (log, UI, tests).
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    /// A command went to the driver.
    CommandSent(Command),
    /// The driver could not take a command.
    CommandRefused {
        /// The command that was not sent.
        command: Command,
        /// Why.
        error: DriverError,
    },
}
