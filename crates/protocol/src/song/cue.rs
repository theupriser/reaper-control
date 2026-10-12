use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A marker that is meant for navigation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct CueInfo {
    /// Identity of the marker.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Position on the project timeline, in seconds.
    pub position: f64,
}
