//! Input.

mod intent;
mod midi;

pub use intent::{Intent, IntentDispatcher, IntentError, IntentRefusal, IntentTranslator};
pub use midi::{
    MidiConfig, MidiConnection, MidiDebounce, MidiDeviceFollower, MidiListener, MidiMessage,
    MidiRouter, MidiSource, MidiSwitch, MidirAnchor, MidirSource,
};
