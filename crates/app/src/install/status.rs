//! How one thing in an installation check stands.

use std::fmt;

/// The result of one check (SPEC §13.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallStatus {
    /// Nothing to do.
    Ok,
    /// The app can put it right.
    Fixable,
    /// The user has to do something; the advice says what.
    Manual,
    /// It could not be determined.
    Unknown,
}

impl fmt::Display for InstallStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Ok => "ok",
            Self::Fixable => "fixable",
            Self::Manual => "manual",
            Self::Unknown => "unknown",
        })
    }
}
