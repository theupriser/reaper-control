use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Where one wizard step stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum WizardStepStatus {
    /// Finished.
    Done,
    /// The step to do now.
    Current,
    /// An earlier step comes first.
    Waiting,
    /// Only the person can fix it; the advice says how.
    NeedsYou,
}
