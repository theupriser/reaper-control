use std::io;

/// The endpoint file could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum EndpointError {
    /// File system problem.
    #[error("endpoint file: {0}")]
    Io(#[from] io::Error),
    /// The file is not a valid endpoint.
    #[error("endpoint file is not valid: {0}")]
    Invalid(#[from] serde_json::Error),
    /// The operating system gave no random bytes.
    #[error("no random bytes for the token: {0}")]
    Random(#[from] getrandom::Error),
}
