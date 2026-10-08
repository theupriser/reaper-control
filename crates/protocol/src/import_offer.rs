use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One v1 setlist that can be brought into the project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ImportOffer {
    /// v1's id of the setlist.
    pub id: String,
    /// Display name.
    pub name: String,
    /// How many songs were found in the project.
    pub found: u32,
    /// Names of the songs the project does not have; they are left out.
    pub missing: Vec<String>,
}
