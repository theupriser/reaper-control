use link::{EndpointError, ServerError};

/// Why the link could not start.
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    /// The server could not listen.
    #[error("link server: {0}")]
    Server(#[from] ServerError),
    /// The endpoint file could not be written.
    #[error("endpoint file: {0}")]
    Endpoint(#[from] EndpointError),
}
