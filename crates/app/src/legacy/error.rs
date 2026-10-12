//! Why a v1 file could not be imported.

/// A failure to read a v1 file.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// The file could not be read.
    #[error("v1 file: {0}")]
    Io(#[from] std::io::Error),
    /// The file is not valid JSON of the v1 shape.
    #[error("v1 file is not understood: {0}")]
    Parse(#[from] serde_json::Error),
    /// The file holds JSON, but neither a list of setlists nor an object with `setlists`.
    #[error("v1 file is not a setlist file")]
    NotASetlistFile,
}
