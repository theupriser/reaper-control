use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CheckId, CheckStatus};

/// One row of the pre-show checklist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CheckItem {
    /// Which check.
    pub id: CheckId,
    /// How it came out.
    pub status: CheckStatus,
    /// What was found or what to do, in plain words.
    pub detail: String,
}
