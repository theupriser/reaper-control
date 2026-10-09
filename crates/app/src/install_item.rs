//! One line of an installation report.

use crate::install_item_kind::InstallItemKind;
use crate::install_status::InstallStatus;

/// One line of an installation report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallItem {
    /// What was checked.
    pub kind: InstallItemKind,
    /// How it stands.
    pub status: InstallStatus,
    /// Plain words for the user: what was found and, unless all is well, what to do.
    pub advice: String,
}

impl InstallItem {
    /// An item with its advice text.
    #[must_use]
    pub fn new(kind: InstallItemKind, status: InstallStatus, advice: impl Into<String>) -> Self {
        Self {
            kind,
            status,
            advice: advice.into(),
        }
    }
}
