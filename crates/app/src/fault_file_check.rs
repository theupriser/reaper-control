//! Reads the `faulted` file the extension leaves behind.

use std::path::PathBuf;

use crate::fault_check::FaultCheck;

/// A fault check that reads the extension's `faulted` file.
#[derive(Debug)]
pub struct FaultFileCheck {
    path: PathBuf,
}

impl FaultFileCheck {
    /// A check on the file at `path`.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl FaultCheck for FaultFileCheck {
    fn reason(&self) -> Option<String> {
        let text = std::fs::read_to_string(&self.path).ok()?;
        let reason = text.trim();
        (!reason.is_empty()).then(|| reason.to_owned())
    }
}

#[cfg(test)]
mod tests;
