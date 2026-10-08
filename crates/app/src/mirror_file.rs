//! The layout of one mirror file.

use protocol::SetlistInfo;
use serde::{Deserialize, Serialize};

/// What a mirror file holds: the setlists of one project, with the layout version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorFile {
    /// Layout version, for later migrations.
    pub schema_version: u32,
    /// The setlists as the project had them.
    pub setlists: Vec<SetlistInfo>,
}
