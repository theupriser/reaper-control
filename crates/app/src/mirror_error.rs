//! Why the setlist mirror could not be used.

/// A failure to write or read the mirror.
#[derive(Debug, thiserror::Error)]
pub enum MirrorError {
    /// The file could not be read or written.
    #[error("setlist mirror: {0}")]
    Io(#[from] std::io::Error),
    /// The file is damaged.
    #[error("setlist mirror is damaged: {0}")]
    Parse(#[from] serde_json::Error),
    /// The project id would not make a safe file name.
    #[error("project id {0:?} cannot be used as a file name")]
    BadProjectId(String),
}
