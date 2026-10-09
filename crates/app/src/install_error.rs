//! Why an installation step did not happen.

use std::path::PathBuf;

use crate::binary_architecture::BinaryArchitecture;

/// Why an installation step did not happen.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("REAPER is running and the extension is already installed; close REAPER first")]
    /// An installed extension cannot be replaced while REAPER has it loaded.
    ReaperIsRunning,
    #[error("this REAPER is built for {found:?}, the extension needs {expected:?}")]
    /// REAPER is built for another processor than the extension.
    WrongArchitecture {
        /// What REAPER is.
        found: BinaryArchitecture,
        /// What the extension needs.
        expected: BinaryArchitecture,
    },
    #[error("the extension that ships with the app is missing at {}", path.display())]
    /// The app's own copy of the extension is gone.
    BundleMissing {
        /// Where it should be.
        path: PathBuf,
    },
    #[error("the extension is not installed")]
    /// There is nothing to remove.
    NotInstalled,
    #[error("could not {action} {}: {source}", path.display())]
    /// The file system refused.
    Io {
        /// What was being done.
        action: &'static str,
        /// The file or folder concerned.
        path: PathBuf,
        /// What the system said.
        source: std::io::Error,
    },
    #[error("could not prepare the extension for loading: {0}")]
    /// Signing or clearing quarantine failed.
    Preparation(String),
}
