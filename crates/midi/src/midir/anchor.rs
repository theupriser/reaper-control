//! The one CoreMIDI client that is made on the main thread.

#[cfg(target_os = "macos")]
use std::sync::Mutex;

#[cfg(target_os = "macos")]
use midir::MidiInput;

/// CoreMIDI keeps one list of devices per process and refreshes it only through the run loop of
/// the thread that made the process's first client. The thread that follows the devices has no
/// run loop, so when it made that first client, a device that appeared later was never seen.
/// Making one client on the main thread, which has a run loop, and keeping it for as long as the
/// app runs, fixes that. Other systems have nothing to anchor.
pub struct MidirAnchor {
    #[cfg(target_os = "macos")]
    _client: Mutex<Option<MidiInput>>,
}

impl MidirAnchor {
    /// Makes the client; call it on the main thread before any other MIDI call.
    #[must_use]
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            _client: Mutex::new(MidiInput::new("Reaper Control anchor").ok()),
        }
    }
}

impl Default for MidirAnchor {
    fn default() -> Self {
        Self::new()
    }
}
