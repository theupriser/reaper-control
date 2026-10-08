//! Why a command did not go out.

use crate::command_refusal::CommandRefusal;
use crate::driver_error::DriverError;

/// Why `CommandBus::dispatch` did not send a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DispatchError {
    /// The driver refused it.
    #[error(transparent)]
    Driver(#[from] DriverError),
    /// The command can never be right.
    #[error(transparent)]
    Invalid(#[from] CommandRefusal),
    /// Too many commands are waiting for an answer from REAPER.
    #[error("too many commands are waiting for REAPER")]
    QueueFull,
}
