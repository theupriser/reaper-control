use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One thing the pre-show checklist looks at (SPEC S-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum CheckId {
    /// The installed extension is the one that ships with the app.
    ExtensionCurrent,
    /// The extension answers.
    Connected,
    /// The project has songs (regions).
    SongsFound,
    /// Every entry of the played setlist points to a song that exists.
    SetlistValid,
    /// MIDI input can be heard.
    MidiPresent,
}
