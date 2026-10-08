use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The settings a person can change on the Settings screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Settings {
    /// Milliseconds inside which an identical command counts as a repeat (0 to 5000).
    pub queue_repeat_window_milliseconds: u32,
    /// Milliseconds after which an unanswered command is reported (500 to 60000).
    pub queue_timeout_milliseconds: u32,
    /// How many commands may wait for an answer (1 to 256).
    pub queue_capacity: u32,
    /// Whether MIDI input is used at all.
    pub midi_enabled: bool,
    /// The only device to listen to; all devices when empty.
    pub midi_device_name: Option<String>,
    /// Only listen to this channel (0 to 15); all channels when empty.
    pub midi_channel: Option<u8>,
    /// Milliseconds inside which the same note counts as one press (0 to 5000).
    pub midi_debounce_milliseconds: u32,
    /// How much the app logs: error, warn, info, debug or trace. Applies after a restart.
    pub log_level: String,
}
