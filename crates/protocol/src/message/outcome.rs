use serde::{Deserialize, Serialize};

/// How a Command ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result")]
pub enum Outcome {
    /// Done.
    Done,
    /// Refused, with a reason for the log.
    Rejected {
        /// Why.
        reason: String,
    },
}
