//! What the installer found when it looked at a REAPER.

use std::fmt;

use crate::install_item::InstallItem;
use crate::install_item_kind::InstallItemKind;
use crate::install_status::InstallStatus;

/// What the installer found when it looked at a REAPER.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstallReport {
    /// One item per thing checked.
    pub items: Vec<InstallItem>,
}

impl InstallReport {
    /// The status of one kind of check, if it was made.
    #[must_use]
    pub fn status_of(&self, kind: InstallItemKind) -> Option<InstallStatus> {
        self.items
            .iter()
            .find(|item| item.kind == kind)
            .map(|item| item.status)
    }

    /// Whether the extension is in place and nothing needs doing.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.items
            .iter()
            .all(|item| item.status == InstallStatus::Ok)
    }

    /// Whether the app can go ahead on its own: nothing waits for the user.
    #[must_use]
    pub fn can_install(&self) -> bool {
        self.items
            .iter()
            .all(|item| item.status != InstallStatus::Manual)
    }
}

impl fmt::Display for InstallReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            writeln!(
                formatter,
                "{}: {} - {}",
                item.kind, item.status, item.advice
            )?;
        }
        Ok(())
    }
}
