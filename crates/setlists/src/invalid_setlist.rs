use thiserror::Error;

/// Why stored data cannot be a setlist.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidSetlist {
    /// A setlist needs a name.
    #[error("setlist name is empty")]
    EmptyName,
    /// Two entries share an id.
    #[error("two entries share an id")]
    DuplicateEntry,
    /// Entry ids ran out.
    #[error("no entry ids left")]
    IdsExhausted,
}
