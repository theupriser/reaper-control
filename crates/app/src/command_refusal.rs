//! Why a command was refused before it left the app.

/// What is wrong with a command, found without asking REAPER.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CommandRefusal {
    /// A seek position that is not a number or is negative.
    #[error("the position must be a number of seconds, zero or more")]
    InvalidPosition,
    /// A setlist without an identity.
    #[error("a setlist needs an id")]
    EmptySetlistId,
    /// A setlist without a name.
    #[error("a setlist needs a name")]
    EmptySetlistName,
}
