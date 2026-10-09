//! What an install would change; empty when the extension is already current. This is the
//! dry-run result, and what `apply` carries out.

use crate::install_action::InstallAction;

/// The steps an install would take, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstallPlan {
    /// The steps, in the order they run.
    pub actions: Vec<InstallAction>,
}

impl InstallPlan {
    /// Whether there is nothing to do.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}
