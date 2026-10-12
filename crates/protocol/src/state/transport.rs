use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What REAPER's transport is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum Transport {
    /// Stopped.
    Stopped,
    /// Playing.
    Playing,
    /// Paused.
    Paused,
    /// Recording.
    Recording,
}
