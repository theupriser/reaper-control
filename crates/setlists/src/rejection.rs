use thiserror::Error;

use crate::Revision;

/// Why an edit was refused. A refused edit changes nothing.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Rejection {
    /// Someone else saved first.
    #[error("setlist is at revision {actual:?}, edit expected {expected:?}")]
    Stale {
        /// What the editor based the edit on.
        expected: Revision,
        /// What the setlist is at.
        actual: Revision,
    },
    /// A setlist needs a name.
    #[error("setlist name is empty")]
    EmptyName,
    /// There is no such entry.
    #[error("no such entry")]
    UnknownEntry,
    /// The index is past the end of the list.
    #[error("position is past the end of the setlist")]
    PositionOutOfRange,
    /// The edit would leave the setlist as it is.
    #[error("edit changes nothing")]
    NoChange,
    /// Entry ids ran out.
    #[error("no entry ids left")]
    IdsExhausted,
}
