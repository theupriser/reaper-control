use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How much a notice matters to the person looking at the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum NoticeLevel {
    /// Something happened that needs no action.
    Info,
    /// Something did not go as asked.
    Warning,
    /// The show may be affected.
    Error,
}
