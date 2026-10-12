//! Whether the installed extension has really started: copying the file does not count, only
//! the extension answering the handshake does (SPEC §13.1 step 6).

use protocol::LinkStatus;

/// The outcome of checking that the installed extension answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstallVerification {
    /// The expected version answered.
    Confirmed,
    /// Connected, but a different version from the one the app ships.
    WrongVersion {
        /// The version that answered.
        found: String,
    },
    /// Nothing answered: REAPER has to be (re)started, or the extension did not load.
    NotConnected,
}

impl InstallVerification {
    /// Judges the link as the UI shows it against the version the app ships.
    #[must_use]
    pub fn from_link(status: &LinkStatus, expected_version: &str) -> Self {
        match status {
            LinkStatus::Connected { extension_version }
                if extension_version == expected_version =>
            {
                Self::Confirmed
            }
            LinkStatus::Connected { extension_version } => Self::WrongVersion {
                found: extension_version.clone(),
            },
            _ => Self::NotConnected,
        }
    }
}
