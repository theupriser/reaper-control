use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The four steps of the first-run wizard, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum WizardStepId {
    /// Find REAPER and check it can take the extension.
    FindReaper,
    /// Copy the extension into REAPER.
    InstallExtension,
    /// REAPER loads extensions only when it starts.
    RestartReaper,
    /// The extension answers the handshake.
    Connect,
}
