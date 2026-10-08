//! Listening to the MIDI devices with `midir`, and following them when they are unplugged or plugged in.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use midir::{MidiInput, MidiInputConnection};

use crate::app_event::AppEvent;
use crate::event_bus::EventBus;
use crate::midi_router::MidiRouter;

const CLIENT_NAME: &str = "Reaper Control";

/// Keeps the MIDI inputs open: the wanted device, or every device when none is named (as v1 did).
/// `midir` reports no hotplug, so a thread looks at the device list every second.
pub struct MidiListener;

impl MidiListener {
    /// Starts the thread; it ends with the process.
    ///
    /// # Errors
    /// The error of the operating system when the thread cannot start.
    pub fn start(
        router: Arc<MidiRouter>,
        wanted: Option<String>,
        events: Arc<EventBus>,
    ) -> std::io::Result<()> {
        std::thread::Builder::new()
            .name("midi-input".into())
            .spawn(move || {
                let mut open: BTreeMap<String, MidiInputConnection<()>> = BTreeMap::new();
                loop {
                    // A listing that failed says nothing about the devices, so nothing is closed.
                    if let Some(names) = device_names() {
                        let before = open.len();
                        open.retain(|name, _| names.contains(name));
                        let mut changed = open.len() != before;
                        for name in names {
                            let wanted = wanted.as_ref().is_none_or(|only| *only == name);
                            if wanted
                                && !open.contains_key(&name)
                                && let Some(connection) = connect(&name, &router)
                            {
                                open.insert(name, connection);
                                changed = true;
                            }
                        }
                        if changed {
                            events.publish(&AppEvent::MidiDevicesChanged {
                                devices: open.keys().cloned().collect(),
                            });
                        }
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            })
            .map(|_| ())
    }
}

/// The names of the input devices present now; `None` when the system cannot be asked.
#[must_use]
pub fn device_names() -> Option<Vec<String>> {
    let input = MidiInput::new(CLIENT_NAME).ok()?;
    Some(
        input
            .ports()
            .iter()
            .filter_map(|port| input.port_name(port).ok())
            .collect(),
    )
}

fn connect(name: &str, router: &Arc<MidiRouter>) -> Option<MidiInputConnection<()>> {
    let input = MidiInput::new(CLIENT_NAME).ok()?;
    let port = input
        .ports()
        .into_iter()
        .find(|port| input.port_name(port).is_ok_and(|found| found == name))?;
    let router = Arc::clone(router);
    input
        .connect(&port, "input", move |_, bytes, ()| router.handle(bytes), ())
        .ok()
}
