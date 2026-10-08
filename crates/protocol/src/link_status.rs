use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Whether the app is talking to the extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum LinkStatus {
    /// No extension answers: REAPER is closed, or the extension is not loaded.
    NotRunning,
    /// Connected and past the handshake.
    Connected {
        /// Version of the extension build.
        extension_version: String,
    },
}
