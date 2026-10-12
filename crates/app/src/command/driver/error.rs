//! Why a driver refused a command.

/// Why a driver could not carry a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DriverError {
    /// There is no connection to the extension.
    #[error("REAPER is not connected")]
    NotConnected,
}
