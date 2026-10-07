use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One place in a setlist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EntryInfo {
    /// Unique within the setlist and never reused.
    #[ts(type = "number")]
    pub id: u64,
    /// The song played here.
    pub song_id: String,
}
