use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::NoticeLevel;

/// A short message for the screen. A newer notice with the same `key` replaces the older one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Notice {
    /// What the notice is about, for example `link` or `command`.
    pub key: String,
    /// How much it matters.
    pub level: NoticeLevel,
    /// What to show.
    pub text: String,
}
