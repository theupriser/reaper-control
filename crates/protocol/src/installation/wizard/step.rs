use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{WizardStepId, WizardStepStatus};

/// One step of the first-run wizard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct WizardStep {
    /// Which step.
    pub id: WizardStepId,
    /// Where it stands.
    pub status: WizardStepStatus,
    /// What to do or what was found, in plain words; empty when there is nothing to add.
    pub advice: String,
}
