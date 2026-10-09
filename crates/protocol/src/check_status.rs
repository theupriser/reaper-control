use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How one check came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum CheckStatus {
    /// In order.
    Passed,
    /// Not in order; the detail says what to do.
    Failed,
    /// Not looked at: it does not apply, or an earlier check failed.
    Skipped,
}
