//! What an installation check looked at.

use std::fmt;

/// What an installation check looked at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallItemKind {
    /// REAPER's resource folder exists.
    ResourceFolder,
    /// The folder can be written to.
    Writable,
    /// REAPER running blocks an update.
    ReaperRunning,
    /// REAPER's processor matches the extension.
    Architecture,
    /// The extension file is installed and current.
    Extension,
}

impl fmt::Display for InstallItemKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ResourceFolder => "resource folder",
            Self::Writable => "writable",
            Self::ReaperRunning => "reaper running",
            Self::Architecture => "architecture",
            Self::Extension => "extension",
        })
    }
}
