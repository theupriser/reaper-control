//! The port through which the app reaches MIDI input devices.

use std::sync::Arc;

use crate::midi_router::MidiRouter;

/// What keeps a device open; dropping it closes the device.
pub type MidiConnection = Box<dyn Send>;

/// The MIDI input devices of the machine; `midir` in production, a fake in tests.
pub trait MidiSource: Send + Sync {
    /// The names of the input devices present now; `None` when the system cannot be asked.
    fn device_names(&self) -> Option<Vec<String>>;

    /// Opens a device and hands every message it sends to `router`; `None` when it cannot be opened.
    fn open(&self, name: &str, router: Arc<MidiRouter>) -> Option<MidiConnection>;
}
