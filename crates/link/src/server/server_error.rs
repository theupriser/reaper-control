use std::io;

/// The server could not start.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// Binding or configuring the listener failed.
    #[error("listener: {0}")]
    Io(#[from] io::Error),
    /// No token could be made.
    #[error(transparent)]
    Endpoint(#[from] crate::EndpointError),
}
