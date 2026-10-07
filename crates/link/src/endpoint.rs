//! The endpoint file: where the extension listens and the token the app must present.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Port and token of a running extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    /// Loopback port.
    pub port: u16,
    /// Shared secret for the Hello.
    pub token: String,
}

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

impl Endpoint {
    /// A fresh 256-bit token from the operating system's random source, as 64 hex digits.
    pub fn new_token() -> Result<String, EndpointError> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes)?;
        Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
    }

    /// Read the endpoint file.
    pub fn read(path: &Path) -> Result<Self, EndpointError> {
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    /// Write the endpoint file in one step, so a reader never sees half of it.
    pub fn write(&self, path: &Path) -> Result<(), EndpointError> {
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, serde_json::to_vec(self)?)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }
}
