use thiserror::Error;

use crate::Revision;

/// Why a repository refused a save or delete.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SaveError {
    /// The stored setlist is not at the revision the caller expected.
    #[error("stored setlist is at {actual:?}, caller expected {expected:?}")]
    Stale {
        /// What the caller based its change on (`None`: expected a new setlist).
        expected: Option<Revision>,
        /// What is stored (`None`: nothing).
        actual: Option<Revision>,
    },
}
