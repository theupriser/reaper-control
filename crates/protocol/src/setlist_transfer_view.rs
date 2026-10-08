use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ImportOffer;

/// What can be moved into the project: the backup copy and the v1 setlists.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SetlistTransferView {
    /// Names of the setlists in the backup copy that the project does not have.
    pub restorable: Vec<String>,
    /// The v1 setlists of this project that are not in it yet.
    pub imports: Vec<ImportOffer>,
    /// Why nothing is offered, when the project is unknown or a file is unusable.
    pub problem: Option<String>,
}
