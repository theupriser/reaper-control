use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A playback setting that can change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum Setting {
    /// "Auto-resume playback".
    Autoplay,
    /// "Count-in when pressing marker".
    CountIn,
}
