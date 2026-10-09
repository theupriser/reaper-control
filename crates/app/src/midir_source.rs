//! The MIDI devices of the machine, through `midir`.

use std::sync::Arc;

use midir::MidiInput;

use crate::midi_router::MidiRouter;
use crate::midi_source::{MidiConnection, MidiSource};

const CLIENT_NAME: &str = "Reaper Control";

/// Real MIDI input.
#[derive(Debug, Default)]
pub struct MidirSource;

impl MidiSource for MidirSource {
    fn device_names(&self) -> Option<Vec<String>> {
        let input = MidiInput::new(CLIENT_NAME).ok()?;
        Some(
            input
                .ports()
                .iter()
                .filter_map(|port| input.port_name(port).ok())
                .collect(),
        )
    }

    fn open(&self, name: &str, router: Arc<MidiRouter>) -> Option<MidiConnection> {
        let input = MidiInput::new(CLIENT_NAME).ok()?;
        let port = input
            .ports()
            .into_iter()
            .find(|port| input.port_name(port).is_ok_and(|found| found == name))?;
        let connection = input
            .connect(&port, "input", move |_, bytes, ()| router.handle(bytes), ())
            .ok()?;
        Some(Box::new(connection))
    }
}
