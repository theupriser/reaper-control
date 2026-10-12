//! MIDI input: finding devices, listening, debouncing and routing messages to intents. Hot-plug
//! follows the device list. Depends on the app core only.

mod debounce;
mod device_follower;
mod fake_source;
mod listener;
mod message;
mod midir;
mod router;
mod source;
mod switch;

pub use debounce::MidiDebounce;
pub use device_follower::MidiDeviceFollower;
pub use fake_source::FakeMidiSource;
pub use listener::MidiListener;
pub use message::MidiMessage;
pub use midir::{MidirAnchor, MidirSource};
pub use router::MidiRouter;
pub use source::{MidiConnection, MidiSource};
pub use switch::MidiSwitch;
