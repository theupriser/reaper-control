//! The endpoint file: where the extension listens and the token the app must present.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

mod endpoint_error;

pub use endpoint_error::EndpointError;

/// Port and token of a running extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    /// Loopback port.
    pub port: u16,
    /// Shared secret for the Hello.
    pub token: String,
    /// Protocol version the extension speaks; 0 when the file has none (an older extension).
    #[serde(default)]
    pub protocol: u32,
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
        let temporary = path.with_extension("temporary");
        fs::write(&temporary, serde_json::to_vec(self)?)?;
        fs::rename(&temporary, path)?;
        Ok(())
    }
}
