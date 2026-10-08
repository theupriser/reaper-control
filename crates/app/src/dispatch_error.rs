//! Why a command did not go out.

use crate::driver_error::DriverError;

/// Why `CommandBus::dispatch` did not send a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DispatchError {
    /// The driver refused it.
    #[error(transparent)]
    Driver(#[from] DriverError),
    /// Too many commands are waiting for an answer from REAPER.
    #[error("too many commands are waiting for REAPER")]
    QueueFull,
}
