use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CheckItem;

/// What the pre-show checklist shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ChecklistView {
    /// The checks in order.
    pub items: Vec<CheckItem>,
    /// Whether no check failed.
    pub ready: bool,
}
