use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::EntryInfo;

/// A setlist as stored in the project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SetlistInfo {
    /// Identity of the setlist.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Raised by one on every accepted edit; saving needs the revision you last saw.
    #[ts(type = "number")]
    pub revision: u64,
    /// The entries in playing order.
    pub entries: Vec<EntryInfo>,
}
